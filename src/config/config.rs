use crate::config::Opts;
use crate::node::AppNode;
use actix_raft::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub bootstrap_hosts: Vec<String>,
    pub is_new_cluster: bool,
    pub rpc_host: String,
    pub rpc_port: u16,
}

impl Config {
    pub fn use_opts(&mut self, opts: &Opts) {}
}
