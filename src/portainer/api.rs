use serde_json::Map;

use super::client::{ClientFactory, Credential, PortainerClient};
use super::commands::{EndpointSelector, FileMapping, InlineEnv};
use super::requests::raw_requests;
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
impl Session {
    pub fn endpoint(&self, selector: &EndpointSelector) -> Res<Endpoint> {
        todo!()
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
    swarm_id: String,
}
impl Endpoint {
    pub fn deploy(
        &self,
        compose: String,
        stack: String,
        inline_vars: Vec<InlineEnv>,
        configs: Vec<FileMapping>,
        secrets: Vec<FileMapping>,
    ) -> Res<Plan> {
        Err(String::from(""))
    }
    pub fn destroy(&self, stack: String, configs: Vec<String>, secrets: Vec<String>) -> Res<Plan> {
        let stacks = raw_requests::list_stacks(Some(self.id), Some(self.swarm_id.to_string()))
            .send(self.client.as_ref())?;

        Err(String::from(""))
    }

    // pub fn create_stack(&self) {}
    // pub fn create_secret(&self) {}
    // pub fn create_config(&self) {}

    // pub fn update_stack(&self) {}

    // pub fn get_stack(&self) {
    //     todo!()
    // }
    // pub fn get_secret(&self) {}
    // pub fn get_config(&self) {}

    // pub fn delete_stack(&self) {
    //     todo!()
    // }
    // pub fn delete_secret(&self) {}
    // pub fn delete_config(&self) {}
}

pub struct Plan {
    definition: PlanDef,
    endpoint: i32,
    client: Box<dyn PortainerClient>,
}
impl Plan {
    pub fn execute(self) -> Action {
        match self.definition {
            PlanDef::Deploy() => Ok(()),
            PlanDef::Destroy {
                stack,
                configs,
                secrets,
            } => Ok(()),
        }
    }
    pub fn print(&self) {
        match &self.definition {
            PlanDef::Deploy() => println!(""),
            PlanDef::Destroy {
                stack,
                configs,
                secrets,
            } => println!(""),
        }
    }
}

enum PlanDef {
    Deploy(),
    Destroy {
        stack: String,
        configs: Map<String, String>,
        secrets: Map<String, String>,
    },
}

enum StackPlan {
    Create {
        compose: String,
        stack: String,
        inline_vars: Vec<InlineEnv>,
        configs: Vec<FileMapping>,
        secrets: Vec<FileMapping>,
    },
    Update {
        compose: String,
        stack_id: u32,
        inline_vars: Vec<InlineEnv>,
        configs: Vec<FileMapping>,
        secrets: Vec<FileMapping>,
    },
}
