use serde::{Deserialize, Serialize};

use super::client::{DefaultClient, PortainerClient};

#[derive(Serialize)]
struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct JwtToken {
    jwt: String,
}

mod raw_requests {
    use crate::portainer::client::PortainerRequest;
    use crate::portainer::requests::*;

    pub fn login(username: &str, password: &str) -> PortainerRequest {
        PortainerRequest::post(
            "/auth",
            Login {
                username: username.to_string(),
                password: password.to_string(),
            },
        )
    }
}

pub trait Requests: PortainerClient {
    fn login(&self, username: &str, password: &str) -> JwtToken {
        self.expect(&raw_requests::login(username, password))
    }
}

impl Requests for DefaultClient {}
