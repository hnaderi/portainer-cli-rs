use std::{collections::HashMap, fs::File};

type URL = String;

pub struct Session {
    credential: SessionCredential,
    address: URL,
}
enum SessionCredential {
    UsernamePassword(String),
    APIToken(String),
}

pub trait SessionManager {
    fn get(&self, name: &str) -> Option<Session>;
    fn save(&self, name: &str, session: &Session);
    fn remove(&self, name: &str);
}

struct LocalSessionManager {
    path: File,
}
impl LocalSessionManager {
    pub fn new() -> LocalSessionManager {
        todo!()
    }
}
impl SessionManager for LocalSessionManager {
    fn get(&self, name: &str) -> Option<Session> {
        todo!()
    }

    fn save(&self, name: &str, session: &Session) {
        todo!()
    }

    fn remove(&self, name: &str) {
        todo!()
    }
}
