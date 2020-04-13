use crate::node::AppNode;
use crate::raft;
use crate::raft::RaftDiscovery;
use crate::raft::RaftSettings;
use crate::rpc::HttpRpcClient;
use actix_raft;
use reqwest::UrlError;
use crate::AppRaft;
use log::{info};
use crate::node;
use crate::actix::Actor;

pub struct RaftBuilder {
    snapshot_dir: Option<String>,
    bootstrap_hosts: Vec<String>,
    rpc_port: u16,
    rpc_host: String,
    discovered_nodes: Vec<AppNode>,
    node_id: actix_raft::NodeId,
}

#[derive(Debug)]
pub enum BuildError {
    BootstrapUrl { err: UrlError },
}

impl RaftBuilder {
    pub fn new(node_id: actix_raft::NodeId) -> Self {
        Self {
            snapshot_dir: None,
            bootstrap_hosts: Vec::new(),
            rpc_port: 5012,
            rpc_host: "0.0.0.0".to_string(),
            node_id,
            discovered_nodes: vec![],
        }
    }

    pub fn snapshot_dir(&mut self, path: &str) -> &mut Self {
        self.snapshot_dir = Some(path.to_string());
        self
    }

    pub fn bootstrap_hosts(&mut self, hosts: Vec<String>) -> &mut Self {
        self.bootstrap_hosts = hosts;
        self
    }

    pub fn rpc_port(&mut self, port: u16) -> &mut Self {
        self.rpc_port = port;
        self
    }

    pub fn rpc_host(&mut self, host: &str) -> &mut Self {
        self.rpc_host = host.to_string();
        self
    }

    pub fn discovery(&self) -> Result<RaftDiscovery, BuildError> {
        let mut urls = Vec::new();

        for url_result in self
            .bootstrap_hosts
            .iter()
            .map(|h| format!("http://{}/rpc", h).parse())
        {
            match url_result {
                Ok(url) => urls.push(url),
                Err(err) => return Err(BuildError::BootstrapUrl { err: err }),
            }
        }

        Ok(RaftDiscovery::new(urls, HttpRpcClient::new()))
    }

    pub fn discovered_nodes(&mut self, nodes: Vec<AppNode>) -> &mut Self {
        self.discovered_nodes = nodes;

        self
    }

    fn activate(&mut self, raft_settings: RaftSettings) -> AppRaft {
        info!("Starting raft");

        let shared_network_state = node::SharedNetworkState::new();

        let membership: actix_raft::messages::MembershipConfig =
            actix_raft::messages::MembershipConfig {
                is_in_joint_consensus: false,
                members: raft_settings.members.clone(),
                non_voters: vec![],
                removing: vec![],
            };

        let node_tracker = node::NodeTracker::new();

        // Start the various actor types and hold on to their addrs.
        let network = raft::AppNetwork::new(
            shared_network_state.clone(),
            raft_settings.node_id,
            node_tracker.start(),
        );
        let storage = raft::AppStorage::new(shared_network_state.clone(), membership);
        let metrics = raft::AppMetrics {};
        let network_addr = network.start();

        let config =
            actix_raft::Config::build(String::from(raft_settings.snapshot_dir.clone()))
                .validate()
                .unwrap();

        AppRaft::new(
            raft_settings.node_id,
            config,
            network_addr.clone(),
            storage.start(),
            metrics.start().recipient(),
        )
    }

    pub fn build(mut self) -> crate::AppRaft {
        let mut members = vec![self.node_id];

        for node in self.discovered_nodes.iter() {
            members.push(node.id);
        }

        let settings = RaftSettings {
            snapshot_dir: self.snapshot_dir.clone().unwrap_or("./".to_string()),
            members: members,
            discovered_nodes: self.discovered_nodes.clone(),
            rpc_port: self.rpc_port,
            rpc_host: self.rpc_host.clone(),
            node_id: self.node_id,
        };

        self.activate(settings)
    }
}
