use std::{fmt::Display, marker::PhantomData, ops::Deref};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as Json;

pub trait PortainerClient {
    fn send(&self, req: &PortainerRequestRaw) -> Res<Json>;
}

pub struct PortainerRequest<O: DeserializeOwned>(PortainerRequestRaw, PhantomData<O>);

impl<O: DeserializeOwned> PortainerRequest<O> {
    pub fn new(request: PortainerRequestRaw) -> PortainerRequest<O> {
        PortainerRequest(request, PhantomData::default())
    }
    pub fn send(&self, client: &dyn PortainerClient) -> Res<O> {
        serde_json::from_value(client.send(&self.0)?).map_err(|x| x.to_string())
    }
}
impl<O: DeserializeOwned> Deref for PortainerRequest<O> {
    type Target = PortainerRequestRaw;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<O: DeserializeOwned> From<PortainerRequest<O>> for PortainerRequestRaw {
    fn from(typed: PortainerRequest<O>) -> Self {
        typed.0
    }
}
impl<O: DeserializeOwned> From<PortainerRequestRaw> for PortainerRequest<O> {
    fn from(req: PortainerRequestRaw) -> Self {
        PortainerRequest::new(req)
    }
}

pub struct PortainerRequestRaw {
    body: Option<Json>,
    path: String,
    queries: Option<Vec<(String, String)>>,
    method: HttpMethod,
}
impl PortainerRequestRaw {
    pub fn get(path: &str) -> PortainerRequestRaw {
        PortainerRequestRaw {
            body: None,
            path: path.to_string(),
            queries: None,
            method: HttpMethod::GET,
        }
    }
    pub fn delete(path: &str) -> PortainerRequestRaw {
        PortainerRequestRaw {
            body: None,
            path: path.to_string(),
            queries: None,
            method: HttpMethod::DELETE,
        }
    }
    pub fn post<I: Serialize>(path: &str, body: I) -> PortainerRequestRaw {
        let body = serde_json::to_value(body).expect("Invalid body!");
        PortainerRequestRaw {
            body: Some(body),
            path: path.to_string(),
            queries: None,
            method: HttpMethod::POST,
        }
    }
    pub fn put<I: Serialize>(path: &str, body: I) -> PortainerRequestRaw {
        let body = serde_json::to_value(body).expect("Invalid body!");
        PortainerRequestRaw {
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
    pub fn with_query_opt<T: Display>(self, key: &str, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.with_query(key, &format!("{}", v))
        } else {
            self
        }
    }
    pub fn with_query_list<T: Display>(self, key: &str, value: Vec<T>) -> Self {
        let mut temp = self;
        for v in value {
            temp = temp.with_query(key, &format!("{}", v));
        }
        temp
    }
    pub fn with_filters<T: Serialize>(self, t: T) -> Self {
        self.with_query(
            "filters",
            &serde_json::to_string(&t).expect("Cannot serialize filters!"),
        )
    }

    pub fn send(&self, client: &dyn PortainerClient) -> Res<Json> {
        client.send(self)
    }
}

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Clone)]
pub enum Credential {
    Public,
    APIToken(String),
    JwtToken(String),
}

use reqwest::blocking::Client as HttpClient;

use super::Res;

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
    fn send(&self, req: &PortainerRequestRaw) -> Res<Json> {
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
        let resp = authorized.send().map_err(|x| x.to_string())?;
        resp.json().map_err(|x| x.to_string())
    }
}

pub struct DefaultClientFactory;
impl ClientFactory for DefaultClientFactory {
    fn build(&self, credential: Credential, server: &str) -> Box<dyn PortainerClient> {
        Box::new(DefaultClient::new(credential, server))
    }
}

pub trait ClientFactory {
    fn build(&self, credential: Credential, server: &str) -> Box<dyn PortainerClient>;
}
