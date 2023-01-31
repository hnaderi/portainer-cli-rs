use std::path::Path;

use crate::portainer::api::{Authentication, Client, Session};
use crate::portainer::client::DefaultClientFactory;
use crate::portainer::commands::{CLICommand, LoginCredential, ServerConfig};
use crate::portainer::session::{LocalSessionManager, SessionManager};

pub struct Application {
    session: Box<dyn SessionManager>,
}

fn readpassword() -> String {
    use std::io::Write;
    print!("Type a password: ");
    std::io::stdout().flush().unwrap();
    rpassword::read_password().expect("Password is required for loging in!")
}

impl Application {
    pub fn new() -> Application {
        let p = Box::from(Path::new(".portainer.json"));
        let lsm = LocalSessionManager::new(p).expect("Invalid session file");
        let session = Box::new(lsm);
        Application { session }
    }

    fn load_session(&self, config: ServerConfig, cl: Client) -> Result<Session, String> {
        match config {
            ServerConfig::InlineLogin {
                address,
                username,
                password,
            } => {
                let password = password.unwrap_or(readpassword());
                cl.authenticate(Authentication::Login { username, password }, &address)
            }
            ServerConfig::InlineToken { address, token } => {
                cl.authenticate(Authentication::APIToken(token), &address)
            }

            ServerConfig::Session(name) => {
                let (auth, url) = self.session.get(&name)?.to_tuple();
                cl.authenticate(auth, &url)
            }
        }
    }

    pub fn handle(&self, command: CLICommand) -> Result<(), String> {
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
                    .authenticate(auth, &address)?
                    .save(self.session.as_ref(), &server)
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
            } => self
                .load_session(server, client)?
                .endpoint(endpoint)?
                .deploy(compose, stack, inline_vars, configs, secrets)?
                .prompt(confirmed),

            CLICommand::Destroy {
                server,
                stacks,
                endpoint,
                confirmed,
                configs,
                secrets,
            } => self
                .load_session(server, client)?
                .endpoint(endpoint)?
                .destroy(stacks, configs, secrets)?
                .prompt(confirmed),

            CLICommand::Logout(name) => self.session.remove(&name),
        }
    }
}
