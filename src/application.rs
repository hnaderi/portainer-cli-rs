use crate::portainer::api::{Authentication, Client, Session2};
use crate::portainer::client::DefaultClientFactory;
use crate::portainer::commands::{CLICommand, LoginCredential, ServerConfig};
use crate::portainer::session::SessionManager;

pub struct Application {
    session: Box<dyn SessionManager>,
}

impl Application {
    pub fn new() -> Application {
        todo!()
    }
    fn load_session(&self, config: ServerConfig, cl: Client) -> Result<Session2, String> {
        match config {
            ServerConfig::InlineLogin {
                address,
                username,
                password,
            } => {
                let password = password.unwrap_or(String::from("")); //TODO readpass
                Ok(cl.authenticate(Authentication::Login { username, password }, &address))
            }
            ServerConfig::InlineToken { address, token } => {
                Ok(cl.authenticate(Authentication::APIToken(token), &address))
            }

            ServerConfig::Session(name) => {
                let _data = self.session.get(&name).ok_or("No such session!")?;

                todo!()
            }
        }
    }

    pub fn handle(&self, command: CLICommand) -> Result<(), String> {
        let not_implemented = Err("Not implemented yet!".to_string());
        let client = Client::new(Box::new(DefaultClientFactory));

        match command {
            CLICommand::Login {
                server,
                address,
                credential,
            } => {
                let auth = match credential {
                    LoginCredential::ByUserPass { username, password } => Authentication::Login {
                        username: username.to_string(),
                        password: password.to_string(),
                    },
                    LoginCredential::ByAPIToken(value) => {
                        Authentication::APIToken(value.to_string())
                    }
                };

                client
                    .authenticate(auth, &address)
                    .save(self.session.as_ref(), &server);
                Ok(())
            }
            CLICommand::Deploy {
                server,
                compose,
                stack,
                endpoint,
                confirmed,
                inline_vars,
                configs,
                secrets,
            } => {
                self.load_session(server, client)?
                    .endpoint(&endpoint)
                    .deploy();

                Ok(())
            }
            CLICommand::Logout(name) => Ok(self.session.remove(&name)),

            _ => not_implemented,
        }
    }
}
