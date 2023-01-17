use serde::{Deserialize, Serialize};

type URL = String;

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    credential: SessionCredential,
    address: URL,
}
impl SessionData {
    pub fn login(url: &str, token: &str) -> SessionData {
        SessionData {
            credential: SessionCredential::UsernamePassword(token.to_string()),
            address: url.to_string(),
        }
    }
    pub fn api(url: &str, token: &str) -> SessionData {
        SessionData {
            credential: SessionCredential::APIToken(token.to_string()),
            address: url.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum SessionCredential {
    UsernamePassword(String),
    APIToken(String),
}

pub trait SessionManager {
    fn get(&self, name: &str) -> Option<SessionData>;
    fn save(&self, name: &str, session: &SessionData);
    fn remove(&self, name: &str);
}

struct LocalSessionManager {}
impl LocalSessionManager {
    pub fn new() -> LocalSessionManager {
        todo!()
    }
}
impl SessionManager for LocalSessionManager {
    fn get(&self, _name: &str) -> Option<SessionData> {
        todo!()
    }

    fn save(&self, _name: &str, _session: &SessionData) {
        todo!()
    }

    fn remove(&self, _name: &str) {
        todo!()
    }
}
