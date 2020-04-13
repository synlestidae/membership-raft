use crate::node;
use crate::rpc;
use crate::rpc::CreateSessionResponse;
use crate::rpc::HttpRpcClient;
use crate::rpc::RpcClient;
use crate::AppRaft;
use actix;
use actix::prelude::Future;
use actix_raft::admin;
use actix_raft::NodeId;
use log::*;

mod create_cluster_request;
mod join_cluster_request;

pub use create_cluster_request::*;
pub use join_cluster_request::*;

pub struct StartupActor {
    node_id: actix_raft::NodeId,
    raft_addr: actix::Addr<AppRaft>,
    http_rpc_client: HttpRpcClient,
}

impl StartupActor {
    pub fn new(
        node_id: actix_raft::NodeId,
        raft_addr: actix::Addr<AppRaft>,
        http_rpc_client: HttpRpcClient,
    ) -> Self {
        Self {
            raft_addr,
            node_id,
            http_rpc_client,
        }
    }
}

impl actix::Actor for StartupActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("StartupActor has started");
    }
}

#[derive(Debug)]
pub struct ClusterConfig {}

#[derive(Debug)]
pub enum StartupRequest {
    NewCluster {
        cluster_config: ClusterConfig,
        config: crate::config::Config,
    },
    ExistingCluster {
        config: crate::config::Config,
    },
}

impl actix::Message for StartupRequest {
    type Result = Result<StartupResponse, StartupErr>;
}

#[derive(Debug)]
pub struct StartupResponse {
    pub leader_node_id: NodeId,
}

#[derive(Debug)]
pub struct StartupErr {}

impl actix::Handler<StartupRequest> for StartupActor {
    type Result = actix::prelude::ResponseFuture<StartupResponse, StartupErr>;

    fn handle(
        &mut self,
        msg: StartupRequest,
        _: &mut <Self as actix::Actor>::Context,
    ) -> Self::Result {
        match msg {
            StartupRequest::NewCluster {
                cluster_config: _,
                config: _,
            } => {
                info!("Starting a new cluster");

                let init_with_config = admin::InitWithConfig {
                    members: vec![self.node_id],
                };

                info!("Initialising with just one member (me!)");

                //let future: Box<dyn ActorFuture<Item=StartupResponse, Error=StartupErr, Actor = Self>> = Box::new(self.raft_addr.send(init_with_config));
                Box::new(
                    self.raft_addr
                        .send(init_with_config)
                        .map(|r| {
                            info!("Successfully added config: {:?}", r);

                            StartupResponse { leader_node_id: 0 } //TODO
                        })
                        .map_err(|err| {
                            error!("Error adding config: {:?}", err);

                            StartupErr {}
                        }),
                )
            }
            StartupRequest::ExistingCluster { config } => {
                let msg = rpc::CreateSessionRequest {
                    new_node: node::AppNode {
                        id: self.node_id,
                        name: config.name.to_string(),
                        host: config.rpc_host.clone(),
                        port: config.rpc_port,
                    },
                };
                let url = reqwest::Url::parse(&format!("http://{}/rpc", config.bootstrap_hosts[0]))
                    .unwrap();
                Box::new(
                    self.http_rpc_client
                        .join_cluster(&url, msg)
                        .map(|r| StartupResponse {
                            leader_node_id: match r {
                                CreateSessionResponse::Success { leader_node_id } => leader_node_id,
                                _ => unimplemented!(),
                            },
                        })
                        .map_err(|_| StartupErr {}),
                )
            }
        }
    }
}
