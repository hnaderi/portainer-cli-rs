use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as Json;

pub trait PortainerClient {
    fn send(&self, req: &PortainerRequest) -> Json;
    fn expect<O: DeserializeOwned>(&self, req: &PortainerRequest) -> O {
        serde_json::from_value(self.send(req)).expect("Invalid response format!")
    }
}

pub struct PortainerRequest {
    body: Option<Json>,
    path: String,
    queries: Option<Vec<(String, String)>>,
    method: HttpMethod,
}
impl PortainerRequest {
    pub fn get(path: &str) -> PortainerRequest {
        PortainerRequest {
            body: None,
            path: path.to_string(),
            queries: None,
            method: HttpMethod::GET,
        }
    }
    pub fn delete(path: &str) -> PortainerRequest {
        PortainerRequest {
            body: None,
            path: path.to_string(),
            queries: None,
            method: HttpMethod::DELETE,
        }
    }
    pub fn post<I: Serialize>(path: &str, body: I) -> PortainerRequest {
        let body = serde_json::to_value(body).expect("Invalid body!");
        PortainerRequest {
            body: Some(body),
            path: path.to_string(),
            queries: None,
            method: HttpMethod::POST,
        }
    }
    pub fn put<I: Serialize>(path: &str, body: I) -> PortainerRequest {
        let body = serde_json::to_value(body).expect("Invalid body!");
        PortainerRequest {
            body: Some(body),
            path: path.to_string(),
            queries: None,
            method: HttpMethod::PUT,
        }
    }

    pub fn with_query(mut self, key: &str, value: &str) -> Self {
        let tpl = (key.to_string(), value.to_string());
        if let Some(qs) = &mut self.queries {
            qs.push(tpl);
        } else {
            self.queries = Some(vec![tpl]);
        }
        self
    }
}

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub enum Credential {
    Public,
    APIToken(String),
    JwtToken(String),
}

use reqwest::{blocking::Client as HttpClient, blocking::RequestBuilder};

pub struct DefaultClient {
    credential: Credential,
    client: HttpClient,
    server: String,
}
impl DefaultClient {
    pub fn new(credential: Credential, server: &str) -> DefaultClient {
        let client = HttpClient::new();
        DefaultClient {
            credential,
            client,
            server: String::from(server),
        }
    }
    fn url_for(&self, path: &str) -> String {
        let mut base = self.server.clone();
        base.push_str(path);
        base
    }
}

impl PortainerClient for DefaultClient {
    fn send(&self, req: &PortainerRequest) -> Json {
        let url = self.url_for(&req.path);
        let client = &self.client;
        let preq = match req.method {
            HttpMethod::GET => client.get(url),
            HttpMethod::POST => client.post(url),
            HttpMethod::PUT => client.put(url),
            HttpMethod::DELETE => client.delete(url),
        };
        let preq = match &req.body {
            Some(value) => preq.json(&value),
            None => preq,
        };

        let authorized = match &self.credential {
            Credential::Public => preq,
            Credential::APIToken(value) => preq.header("x-api-key", value),
            Credential::JwtToken(value) => preq.header("Authorization", format!("Bearer {value}")),
        };
        let resp = authorized.send();
        match resp {
            Err(_err) => panic!("Failure!"),
            Ok(rrr) => rrr.json().expect("Not a valid json!"),
        }
    }
}
