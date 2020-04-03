use actix_raft::NodeId;
use crate::node::AppNode;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub bootstrap_nodes: Vec<AppNode>,
    pub node_id: Option<NodeId>,
    pub webserver: WebserverConfig
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WebserverConfig {
    pub port: u16
}
