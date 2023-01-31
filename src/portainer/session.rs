use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::Path,
};

use serde::{Deserialize, Serialize};

use super::{api::Authentication, Action, Res};

type URL = String;

#[derive(Serialize, Deserialize, Clone)]
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
    pub fn to_tuple(&self) -> (Authentication, URL) {
        (
            match &self.credential {
                SessionCredential::APIToken(value) => Authentication::APIToken(value.to_string()),
                SessionCredential::UsernamePassword(value) => {
                    Authentication::Jwt(value.to_string())
                }
            },
            self.address.to_string(),
        )
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum SessionCredential {
    UsernamePassword(String),
    APIToken(String),
}

#[derive(Serialize, Deserialize)]
struct SessionStorage {
    sessions: HashMap<String, SessionData>,
}

pub trait SessionManager {
    fn get(&self, name: &str) -> Res<SessionData>;
    fn save(&self, name: &str, session: &SessionData) -> Action;
    fn remove(&self, name: &str) -> Action;
}

pub struct LocalSessionManager {
    path: Box<Path>,
}
impl LocalSessionManager {
    pub fn new(path: Box<Path>) -> Res<LocalSessionManager> {
        if path.exists() {
            File::open(&path)
        } else {
            File::create(&path)
        }
        .map_err(|err| err.to_string())?; //TODO model error

        Ok(LocalSessionManager { path })
    }

    fn load(&self) -> Res<SessionStorage> {
        let content = fs::read_to_string(&self.path).map_err(|err| err.to_string())?; //TODO model error
        serde_json::from_str::<SessionStorage>(&content).map_err(|err| err.to_string())
        //TODO model error
    }
    fn store(&self, store: SessionStorage) -> Action {
        let content = serde_json::to_string(&store).map_err(|err| err.to_string())?;
        //TODO model error
        fs::write(&self.path, content).map_err(|err| err.to_string())
    }
}
impl SessionManager for LocalSessionManager {
    fn get(&self, name: &str) -> Res<SessionData> {
        let ss = self.load()?;
        let ses = ss.sessions.get(name).ok_or_else(|| "".to_string())?;
        Ok(ses.clone())
    }

    fn save(&self, _name: &str, _session: &SessionData) -> Action {
        todo!()
    }

    fn remove(&self, _name: &str) -> Action {
        todo!()
    }
}
