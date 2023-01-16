use crate::portainer::client as pctl;
use crate::portainer::commands::CLICommand;
use crate::portainer::requests::Requests;
use crate::portainer::session::SessionManager;

pub struct Application {
    session: Box<dyn SessionManager>,
}

impl Application {
    pub fn new() -> Application {
        todo!()
    }

    pub fn handle(&self, command: &CLICommand) -> Result<(), String> {
        match command {
            CLICommand::Login {
                server,
                address,
                credential: _,
            } => {
                let client = pctl::DefaultClient::new(pctl::Credential::Public, "");
                let _res = client.login("username", "password");

                println!("login to server:{:?} at: {:?}", server, address);
                Ok(())
            }
            _ => Err("Not implemented yet!".to_string()),
        }
    }
}
