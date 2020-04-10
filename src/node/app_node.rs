use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use reqwest::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppNode {
    pub id: u64,
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl AppNode {
    pub fn rpc_url(&self) -> Url {
        format!("http://{}:{}/rpc", self.host, self.port).parse().unwrap()
    }
}
