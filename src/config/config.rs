use crate::node::AppNode;
use actix_raft::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub bootstrap_hosts: Vec<String>,
    pub webserver: WebserverConfig,
    pub new_cluster: bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WebserverConfig {
    pub host: String,
    pub port: u16,
}
