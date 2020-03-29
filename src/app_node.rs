use std::net::IpAddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppNode {
    pub id: u64,
    pub name: String,
    pub address: IpAddr,
    pub port: u16
}

/// Your application's network interface actor.
pub struct AppNetwork {
    nodes: Vec<AppNode>
}
