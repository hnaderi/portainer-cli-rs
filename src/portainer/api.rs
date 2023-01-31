use std::fs;
use std::path::Path;

use super::client::{ClientFactory, Credential, PortainerClient};
use super::commands::{EndpointSelector, FileMapping, InlineEnv};
use super::requests::{self, raw_requests, Config, Secret, Stack};
use super::session::{SessionData, SessionManager};
use super::{Action, Res};

pub struct Client {
    builder: Box<dyn ClientFactory>,
}
impl Client {
    pub fn new(builder: Box<dyn ClientFactory>) -> Client {
        Client { builder }
    }
    pub fn authenticate(&self, auth: Authentication, url: &str) -> Res<Session> {
        let credential = match &auth {
            Authentication::Jwt(token) => Credential::JwtToken(token.clone()),
            Authentication::APIToken(token) => Credential::APIToken(token.clone()),
            _ => Credential::Public,
        };

        let mut client = self.builder.build(credential.clone(), url);

        if let Authentication::Login { username, password } = auth {
            let token = Client::login(client, &username, &password)?;
            client = self.builder.build(Credential::JwtToken(token), url);
        }

        Ok(Session {
            client,
            credential,
            url: url.to_string(),
        })
    }

    fn login(client: Box<dyn PortainerClient>, username: &str, password: &str) -> Res<String> {
        raw_requests::login(username, password)
            .send(client.as_ref())
            .map(|r| r.value)
    }
}

pub enum Authentication {
    Login { username: String, password: String },
    Jwt(String),
    APIToken(String),
}

pub struct Session {
    client: Box<dyn PortainerClient>,
    credential: Credential,
    url: String,
}

fn assert_selected(all: Vec<requests::Endpoint>) -> Res<i32> {
    if all.len() == 1 {
        Ok(all.get(0).unwrap().id())
    } else {
        Err(format!(
            "Must select exactly one endpoint, but selected {}",
            all.len()
        ))
    }
}
impl Session {
    pub fn endpoint(self, selector: EndpointSelector) -> Res<Endpoint> {
        let client = self.client.as_ref();

        let id = match selector {
            EndpointSelector::ById(id) => id.to_owned(),
            EndpointSelector::ByName(name) => assert_selected(
                raw_requests::list_endpoints(vec![], Some(name.to_string())).send(client)?,
            )?,
            EndpointSelector::ByTagIds(tag_ids) => {
                assert_selected(raw_requests::list_endpoints(tag_ids, None).send(client)?)?
            }
            EndpointSelector::ByTags(tags) => {
                let tag_ids = raw_requests::list_tags()
                    .send(client)?
                    .iter()
                    .filter(|t| tags.contains(&t.name))
                    .map(|t| t.tagged_endpoints())
                    .reduce(|a, b| a.intersection(&b).map(|x| *x).collect())
                    .ok_or("Tags must select a unique endpoint")?
                    .into_iter()
                    .collect();
                assert_selected(raw_requests::list_endpoints(tag_ids, None).send(client)?)?
            }
        };

        Ok(Endpoint {
            client: self.client,
            id,
        })
    }
    pub fn save(&self, session: &dyn SessionManager, name: &str) -> Action {
        let data = match &self.credential {
            Credential::APIToken(value) => Some(SessionData::api(&self.url, value)),
            Credential::JwtToken(value) => Some(SessionData::login(&self.url, value)),
            _ => None,
        };

        let d = data.ok_or("Saving username and password is not supported!".to_string())?;

        session.save(name, &d)
    }
}

pub struct Endpoint {
    client: Box<dyn PortainerClient>,
    id: i32,
}
impl Endpoint {
    pub fn deploy(
        self,
        compose: String,
        stack: String,
        inline_vars: Vec<InlineEnv>,
        configs: Vec<FileMapping>,
        secrets: Vec<FileMapping>,
    ) -> Res<Plan> {
        let client = self.client.as_ref();
        let swarm_id = raw_requests::get_endpoint_info(self.id)
            .send(client)?
            .swarm
            .id;

        let all_stacks =
            raw_requests::list_stacks(Some(self.id), Some(swarm_id.to_string())).send(client)?;

        let stack_plan = match all_stacks.iter().find(|s| s.name == stack) {
            None => StackPlan::Create {
                name: stack,
                swarm_id,
            },
            Some(s) => StackPlan::Update(s.id),
        };
        let definition = PlanDef::Deploy {
            stack_plan,
            compose,
            inline_vars,
            configs,
            secrets,
        };

        Ok(Plan {
            definition,
            endpoint: self.id,
            client: self.client,
        })
    }

    pub fn destroy(
        self,
        stacks: Vec<String>,
        configs: Vec<String>,
        secrets: Vec<String>,
    ) -> Res<Plan> {
        let client = self.client.as_ref();

        let all_stacks = raw_requests::list_stacks(Some(self.id), None).send(client)?;

        let stacks = all_stacks
            .iter()
            .filter(|s| stacks.contains(&s.name))
            .map(|s| s.clone())
            .collect();

        let configs = raw_requests::list_configs(self.id, None, Some(configs)).send(client)?;
        let secrets = raw_requests::list_secrets(self.id, None, Some(secrets)).send(client)?;

        let definition = PlanDef::Destroy {
            stacks,
            configs,
            secrets,
        };

        Ok(Plan {
            definition,
            endpoint: self.id,
            client: self.client,
        })
    }
}

pub struct Plan {
    definition: PlanDef,
    endpoint: i32,
    client: Box<dyn PortainerClient>,
}
impl Plan {
    fn read(path: Box<Path>) -> Res<String> {
        fs::read_to_string(path).map_err(|x| x.to_string())
    }

    pub fn execute(self) -> Action {
        match self.definition {
            PlanDef::Deploy {
                stack_plan,
                compose,
                inline_vars,
                configs,
                secrets,
            } => {
                let client = self.client.as_ref();

                for FileMapping(name, path) in configs {
                    let content = Plan::read(path)?;
                    raw_requests::create_config(self.endpoint, name, content).send(client)?;
                }
                for FileMapping(name, path) in secrets {
                    let content = Plan::read(path)?;
                    raw_requests::create_secret(self.endpoint, name, content).send(client)?;
                }

                let env = inline_vars
                    .into_iter()
                    .map(|InlineEnv(key, value)| (key, value))
                    .collect();

                match stack_plan {
                    StackPlan::Create { name, swarm_id } => {
                        raw_requests::create_stacks(self.endpoint, swarm_id, name, compose, env)
                            .send(self.client.as_ref())
                    }
                    StackPlan::Update(stack_id) => {
                        raw_requests::update_stacks(self.endpoint, stack_id, compose, env, true)
                            .send(client)
                    }
                }?;

                Ok(())
            }
            PlanDef::Destroy {
                stacks,
                configs,
                secrets,
            } => {
                let client = self.client.as_ref();

                for stack in stacks {
                    raw_requests::delete_stack(stack.id).send(client)?;
                }
                for config in configs {
                    raw_requests::delete_config(self.endpoint, config.id).send(client)?;
                }
                for secret in secrets {
                    raw_requests::delete_secret(self.endpoint, secret.id).send(client)?;
                }

                Ok(())
            }
        }
    }

    pub fn prompt(self, confirmed: bool) -> Action {
        if confirmed {
            self.execute()
        } else {
            self.print();
            for line in std::io::stdin().lines() {
                let line = line.map_err(|x| x.to_string()).map(|s| s.to_lowercase())?;
                if line == "yes" {
                    self.execute()?;
                    break;
                } else if line == "no" {
                    break;
                } else {
                    println!("You must answer 'yes' or 'no'");
                    continue;
                }
            }
            Ok(())
        }
    }

    pub fn print(&self) {
        match &self.definition {
            PlanDef::Deploy {
                stack_plan,
                compose: _,
                inline_vars: _,
                configs,
                secrets,
            } => {
                println!("Deploy plan:");
                match stack_plan {
                    StackPlan::Create { name, swarm_id } => println!(
                        "Create a new stack with name {} on swarm cluster '{}'",
                        name, swarm_id
                    ),
                    StackPlan::Update(id) => println!("Update existing stack with id {}", id),
                };

                for FileMapping(name, _) in configs {
                    println!("Try to create a new config with name {}", name);
                }

                for FileMapping(name, _) in secrets {
                    println!("Try to create a new secret with name {}", name);
                }
            }
            PlanDef::Destroy {
                stacks,
                configs,
                secrets,
            } => {
                println!("Destroy plan:");
                for stack in stacks {
                    println!("Stack '{}' will be removed. (id: {})", stack.name, stack.id);
                }
                for config in configs {
                    println!(
                        "Config '{}' will be removed. (id: {})",
                        config.name(),
                        config.id
                    );
                }
                for secret in secrets {
                    println!(
                        "Secret '{}' will be removed. (id: {})",
                        secret.name(),
                        secret.id
                    );
                }
            }
        }
    }
}

enum PlanDef {
    Deploy {
        stack_plan: StackPlan,
        compose: String,
        inline_vars: Vec<InlineEnv>,
        configs: Vec<FileMapping>,
        secrets: Vec<FileMapping>,
    },
    Destroy {
        stacks: Vec<Stack>,
        configs: Vec<Config>,
        secrets: Vec<Secret>,
    },
}

enum StackPlan {
    Create { name: String, swarm_id: String },
    Update(i32),
}
