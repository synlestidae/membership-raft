use crate::messages;
use crate::node;
use crate::raft;
use crate::raft::RaftFut;
use crate::rpc;
use crate::rpc::RpcClient;
use crate::AppRaft;
use actix;
use actix_raft;
use actix_raft::admin;
use log::{error, info, warn};
use crate::futures::Future;
use crate::actix::Actor;

pub struct Raft {
    raft_settings: raft::RaftSettings,
    http_rpc_client: rpc::HttpRpcClient,
    raft_addr: Option<actix::Addr<AppRaft>>,
}

impl Raft {
    pub fn new(
        raft_settings: raft::RaftSettings,
        http_rpc_client: rpc::HttpRpcClient,
    ) -> Self {
        Self {
            raft_settings,
            http_rpc_client,
            raft_addr: None,
        }
    }

    pub fn activate(&mut self) {
        info!("Starting raft");

        let shared_network_state = node::SharedNetworkState::new();

        let membership: actix_raft::messages::MembershipConfig =
            actix_raft::messages::MembershipConfig {
                is_in_joint_consensus: false,
                members: self.raft_settings.members.clone(),
                non_voters: vec![],
                removing: vec![],
            };

        let node_tracker = node::NodeTracker::new();

        // Start the various actor types and hold on to their addrs.
        let network = raft::AppNetwork::new(
            shared_network_state.clone(),
            self.raft_settings.node_id,
            node_tracker.start(),
        );
        let storage = raft::AppStorage::new(shared_network_state.clone(), membership);
        let metrics = raft::AppMetrics {};
        let network_addr = network.start();

        let config =
            actix_raft::Config::build(String::from(self.raft_settings.snapshot_dir.clone()))
                .validate()
                .unwrap();

        let app_raft = AppRaft::new(
            self.raft_settings.node_id,
            config,
            network_addr.clone(),
            storage.start(),
            metrics.start().recipient(),
        );

        let port = self.raft_settings.rpc_port;

        self.raft_addr = Some(app_raft.start());

        info!("Raft has now started");
    }
}

impl actix::Actor for Raft {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Raft actor now STARTED");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        warn!("Raft is stopping");

        actix::Running::Continue
    }
}

impl actix::Handler<messages::CreateClusterRequest> for Raft {
    type Result = RaftFut<(), ()>; //actix::ResponseActFuture<Self, (), ()>;

    fn handle(
        &mut self,
        msg: messages::CreateClusterRequest,
        _: &mut Self::Context,
    ) -> Self::Result {
        info!("Creating a cluster: {:?}", msg);

        match self.raft_addr {
            Some(ref mut addr) => {
                    let request = addr.send(admin::InitWithConfig {
                        members: vec![msg.this_node.id],
                    });

                    RaftFut(Box::new(request.then(|res| RaftFut(match res {
                        Ok(Ok(result)) => Box::new(futures::future::ok(result)),
                        _ => Box::new(futures::future::err(()))
                    }))))
            }
            None => {
                error!("Raft has not initialized with an address");

                RaftFut(Box::new(futures::future::err(())))
            }
        }
    }
}

impl actix::Handler<messages::JoinClusterRequest> for Raft {
    type Result = RaftFut<rpc::CreateSessionResponse, rpc::RpcError>;

    fn handle(&mut self, msg: messages::JoinClusterRequest, _: &mut Self::Context) -> Self::Result {
        info!("Joining a cluster: {:?}", msg);

        RaftFut(Box::new(self.http_rpc_client.join_cluster(
            &msg.this_node.rpc_url(),
            rpc::CreateSessionRequest {
                new_node: msg.this_node,
            },
        )))
    }
}
