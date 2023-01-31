use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct JwtToken {
    #[serde(rename = "jwt")]
    pub value: String,
}

#[derive(Deserialize)]
pub struct Tag {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Endpoints")]
    endpoints: HashMap<i32, bool>,
}
impl Tag {
    pub fn tagged_endpoints(&self) -> HashSet<i32> {
        self.endpoints
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| *k)
            .collect()
    }
}

#[derive(Deserialize)]
pub struct SwarmInfo {
    #[serde(rename = "Cluster")]
    pub cluster: String,
    #[serde(rename = "ID")]
    pub id: String,
}
#[derive(Deserialize)]
pub struct EndpointInfo {
    #[serde(rename = "Swarm")]
    pub swarm: SwarmInfo,
}

#[derive(Deserialize)]
struct ConfigSecretSpec {
    #[serde(rename = "name")]
    name: String,
}

#[derive(Serialize)]
struct ConfigSecretFilter {
    #[serde(rename = "id")]
    id: Option<String>,
    #[serde(rename = "names")]
    names: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "spec")]
    spec: ConfigSecretSpec,
}
impl Config {
    pub fn name(&self) -> &String {
        &self.spec.name
    }
}

#[derive(Deserialize)]
pub struct Secret {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "spec")]
    spec: ConfigSecretSpec,
}
impl Secret {
    pub fn name(&self) -> &String {
        &self.spec.name
    }
}

#[derive(Deserialize, Clone)]
pub struct Stack {
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "SwarmId")]
    pub swarm_id: Option<String>,
    #[serde(rename = "EndpointId")]
    pub endpoint_id: String,
}
#[derive(Serialize)]
struct StackFilter {
    #[serde(rename = "SwarmID")]
    swarm_id: Option<String>,
    #[serde(rename = "EndpointID")]
    endpoint_id: Option<i32>,
}

#[derive(Serialize)]
struct StackUpdate {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "StackFileContent")]
    content: String,
    #[serde(rename = "Env")]
    env: HashMap<String, String>,
    #[serde(rename = "Prune")]
    prune: bool,
}
#[derive(Serialize)]
struct StackCreate {
    #[serde(rename = "SwarmID")]
    swarm_id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "StackFileContent")]
    content: String,
    #[serde(rename = "Env")]
    env: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct Endpoint {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "TagIds")]
    tag_ids: Vec<String>,
}
impl Endpoint {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Serialize)]
struct ConfigSecretRequest {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Labels")]
    labels: HashMap<String, String>,
    #[serde(rename = "Data")]
    content: String,
}

fn base64(data: String) -> String {
    use base64::engine::general_purpose;
    use base64::Engine as _;
    general_purpose::STANDARD.encode(data)
}

pub mod raw_requests {
    use crate::portainer::client::{PortainerRequest, PortainerRequestRaw};
    use crate::portainer::requests::*;

    pub fn login(username: &str, password: &str) -> PortainerRequest<JwtToken> {
        PortainerRequestRaw::post(
            "/auth",
            Login {
                username: username.to_string(),
                password: password.to_string(),
            },
        )
        .into()
    }

    pub fn list_tags() -> PortainerRequest<Vec<Tag>> {
        PortainerRequestRaw::get("/tags").into()
    }

    pub fn get_endpoint_info(id: i32) -> PortainerRequest<EndpointInfo> {
        PortainerRequestRaw::get(&format!("/endpoints/{}/docker/info", id)).into()
    }

    pub fn create_secret(endpoint: i32, name: String, content: String) -> PortainerRequestRaw {
        PortainerRequestRaw::post(
            &format!("/endpoints/{}/docker/secrets/create", endpoint),
            ConfigSecretRequest {
                name,
                labels: HashMap::new(),
                content: base64(content),
            },
        )
    }
    pub fn delete_secret(endpoint: i32, id: String) -> PortainerRequestRaw {
        PortainerRequestRaw::delete(&format!("/endpoints/{}/docker/secrets/{}", endpoint, id))
    }

    pub fn list_secrets(
        endpoint: i32,
        id: Option<String>,
        names: Option<Vec<String>>,
    ) -> PortainerRequest<Vec<Secret>> {
        PortainerRequestRaw::get(&format!("/endpoints/{}/docker/secrets", endpoint))
            .with_filters(ConfigSecretFilter { id, names })
            .into()
    }

    pub fn create_config(endpoint: i32, name: String, content: String) -> PortainerRequestRaw {
        PortainerRequestRaw::post(
            &format!("/endpoints/{}/docker/configs/create", endpoint),
            ConfigSecretRequest {
                name,
                labels: HashMap::new(),
                content: base64(content),
            },
        )
    }
    pub fn delete_config(endpoint: i32, id: String) -> PortainerRequestRaw {
        PortainerRequestRaw::delete(&format!("/endpoints/{}/docker/configs/{}", endpoint, id))
    }

    pub fn list_configs(
        endpoint: i32,
        id: Option<String>,
        names: Option<Vec<String>>,
    ) -> PortainerRequest<Vec<Config>> {
        PortainerRequestRaw::get(&format!("/endpoints/{}/docker/configs", endpoint))
            .with_filters(ConfigSecretFilter { id, names })
            .into()
    }

    pub fn list_endpoints(
        tag_ids: Vec<i32>,
        name: Option<String>,
    ) -> PortainerRequest<Vec<Endpoint>> {
        PortainerRequestRaw::get("/endpoints")
            .with_query_list("tagIds", tag_ids)
            .with_query_opt("name", name)
            .into()
    }
    pub fn get_endpoint(id: i32) -> PortainerRequest<Endpoint> {
        PortainerRequestRaw::get(&format!("/endpoints/{}", id)).into()
    }

    pub fn list_stacks(
        endpoint_id: Option<i32>,
        swarm_id: Option<String>,
    ) -> PortainerRequest<Vec<Stack>> {
        PortainerRequestRaw::get("/stacks")
            .with_filters(StackFilter {
                endpoint_id,
                swarm_id,
            })
            .into()
    }
    pub fn get_stack(id: i32) -> PortainerRequest<Stack> {
        PortainerRequestRaw::get(&format!("/stacks/{}", id)).into()
    }
    pub fn update_stacks(
        endpoint_id: i32,
        id: i32,
        content: String,
        env: HashMap<String, String>,
        prune: bool,
    ) -> PortainerRequestRaw {
        PortainerRequestRaw::put(
            "/stacks",
            StackUpdate {
                id,
                content,
                env,
                prune,
            },
        )
        .with_query("endpointId", &endpoint_id.to_string())
    }
    pub fn create_stacks(
        endpoint_id: i32,
        swarm_id: String,
        name: String,
        content: String,
        env: HashMap<String, String>,
    ) -> PortainerRequestRaw {
        PortainerRequestRaw::post(
            "/stacks",
            StackCreate {
                swarm_id,
                name,
                content,
                env,
            },
        )
        .with_query("endpointId", &format!("{}", endpoint_id))
        .with_query("method", "string")
        .with_query("type", "1")
    }
    pub fn delete_stack(id: i32) -> PortainerRequestRaw {
        PortainerRequestRaw::delete(&format!("/stacks/{}", id))
    }
}
