use std::net::IpAddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppNode {
    pub id: u64,
    pub name: String,
    pub address: IpAddr,
    pub port: u16
}
