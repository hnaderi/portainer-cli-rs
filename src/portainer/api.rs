use super::client::{self, ClientFactory, Credential, PortainerClient};
use super::commands::EndpointSelector;
use super::requests::{raw_requests, JwtToken};
use super::session::{SessionData, SessionManager};

pub struct Client {
    builder: Box<dyn ClientFactory>,
}
impl Client {
    pub fn new(builder: Box<dyn ClientFactory>) -> Client {
        Client { builder }
    }
    pub fn authenticate(&self, auth: Authentication, url: &str) -> Session2 {
        let credential = match &auth {
            Authentication::Jwt(token) => Credential::JwtToken(token.clone()),
            Authentication::APIToken(token) => Credential::APIToken(token.clone()),
            _ => Credential::Public,
        };

        let mut client = self.builder.build(credential.clone(), url);

        if let Authentication::Login { username, password } = auth {
            let token = Client::login(client, &username, &password);
            client = self.builder.build(Credential::JwtToken(token), url);
        }

        Session2 {
            client,
            credential,
            url: url.to_string(),
        }
    }

    fn login(client: Box<dyn PortainerClient>, username: &str, password: &str) -> String {
        client::expect::<JwtToken>(client.as_ref(), &raw_requests::login(username, password)).value
    }
}

pub enum Authentication {
    Login { username: String, password: String },
    Jwt(String),
    APIToken(String),
}

pub struct Session2 {
    client: Box<dyn PortainerClient>,
    credential: Credential,
    url: String,
}
impl Session2 {
    pub fn endpoint(&self, selector: &EndpointSelector) -> Endpoint {
        todo!()
    }
    pub fn save(&self, session: &dyn SessionManager, name: &str) {
        let data = match &self.credential {
            Credential::APIToken(value) => Some(SessionData::api(&self.url, value)),
            Credential::JwtToken(value) => Some(SessionData::login(&self.url, value)),
            _ => None,
        };

        data.map(|d| session.save(name, &d));
    }
}

pub struct Endpoint {
    client: Box<dyn PortainerClient>,
    id: i32,
}
impl Endpoint {
    pub fn deploy(&self) {}
    pub fn destroy(&self) {}

    pub fn create_stack(&self) {}
    pub fn create_secret(&self) {}
    pub fn create_config(&self) {}

    pub fn update_stack(&self) {}

    pub fn get_stack(&self) {
        todo!()
    }
    pub fn get_secret(&self) {}
    pub fn get_config(&self) {}

    pub fn delete_stack(&self) {
        todo!()
    }
    pub fn delete_secret(&self) {}
    pub fn delete_config(&self) {}
}
