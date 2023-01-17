use std::collections::HashMap;

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
    id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Endpoints")]
    endpoints: HashMap<i32, bool>,
}

#[derive(Deserialize)]
struct SwarmInfo {
    #[serde(rename = "Cluster")]
    cluster: String,
    #[serde(rename = "ID")]
    id: String,
}
#[derive(Deserialize)]
pub struct EndpointInfo {
    #[serde(rename = "Swarm")]
    swarm: SwarmInfo,
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
    id: String,
    #[serde(rename = "spec")]
    spec: ConfigSecretSpec,
}

#[derive(Deserialize)]
pub struct Secret {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "spec")]
    spec: ConfigSecretSpec,
}

#[derive(Deserialize)]
pub struct Stack {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "SwarmId")]
    swarm_id: Option<String>,
    #[serde(rename = "EndpointId")]
    endpoint_id: String,
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
    fn name(&self) -> &str {
        &self.name
    }
    fn id(&self) -> i32 {
        self.id
    }
}

pub mod raw_requests {
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

    pub fn list_tags() -> PortainerRequest {
        PortainerRequest::get("/tags")
    }

    pub fn get_endpoint_info(id: i32) -> PortainerRequest {
        PortainerRequest::get(&format!("/endpoints/{}/docker/info", id))
    }

    pub fn list_secrets(
        endpoint: i32,
        id: Option<String>,
        names: Option<Vec<String>>,
    ) -> PortainerRequest {
        PortainerRequest::get(&format!("/endpoints/{}/docker/secrets", endpoint))
            .with_filters(ConfigSecretFilter { id, names })
    }
    pub fn list_configs(
        endpoint: i32,
        id: Option<String>,
        names: Option<Vec<String>>,
    ) -> PortainerRequest {
        PortainerRequest::get(&format!("/endpoints/{}/docker/configs", endpoint))
            .with_filters(ConfigSecretFilter { id, names })
    }

    pub fn list_endpoints(tag_ids: Vec<i32>, name: Option<String>) -> PortainerRequest {
        PortainerRequest::get("/endpoints")
            .with_query_list("tagIds", tag_ids)
            .with_query_opt("name", name)
    }
    pub fn get_endpoint(id: i32) -> PortainerRequest {
        PortainerRequest::get(&format!("/endpoints/{}", id))
    }

    pub fn list_stacks(endpoint_id: Option<i32>, swarm_id: Option<String>) -> PortainerRequest {
        PortainerRequest::get("/stacks").with_filters(StackFilter {
            endpoint_id,
            swarm_id,
        })
    }
    pub fn get_stack(id: i32) -> PortainerRequest {
        PortainerRequest::get(&format!("/stacks/{}", id))
    }
    pub fn update_stacks(
        endpoint_id: i32,
        id: i32,
        content: String,
        env: HashMap<String, String>,
        prune: bool,
    ) -> PortainerRequest {
        PortainerRequest::put(
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
    ) -> PortainerRequest {
        PortainerRequest::post(
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
}
