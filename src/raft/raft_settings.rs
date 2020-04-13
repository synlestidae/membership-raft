use crate::actix::Actor;
use crate::node;
use crate::node::AppNode;
use crate::raft;
use crate::raft::RaftDiscovery;
use crate::rpc::HttpRpcClient;
use crate::AppRaft;
use crate::NodeTracker;
use actix_raft;
use actix_raft::messages;
use log::info;
use reqwest::UrlError;

pub struct RaftSettings {
    pub snapshot_dir: String,
    pub members: Vec<actix_raft::NodeId>,
    pub discovered_nodes: Vec<AppNode>,
    pub rpc_port: u16,
    pub rpc_host: String,
    pub node_id: actix_raft::NodeId,
}
