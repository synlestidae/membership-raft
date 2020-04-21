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
use tokio::time::Delay;
use std::time::Duration;

pub struct Raft {
    http_rpc_client: rpc::HttpRpcClient,
    raft_addr: actix::Addr<AppRaft>,
}

impl Raft {
    pub fn new(
        http_rpc_client: rpc::HttpRpcClient,
        raft_addr: actix::Addr<AppRaft>,
    ) -> Self {
        Self {
            raft_addr,
            http_rpc_client
        }
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

impl Raft {
    pub fn join_cluster(&self, msg: messages::JoinClusterRequest) -> RaftFut<rpc::CreateSessionResponse, rpc::RpcError> {
        info!("Joining a cluster: {:?}", msg);

        RaftFut(Box::new(self.http_rpc_client.join_cluster(
            &msg.this_node.rpc_url(),
            rpc::CreateSessionRequest {
                new_node: msg.this_node,
            },
        )))
    }

    pub fn create_cluster(&self, msg: messages::CreateClusterRequest) -> RaftFut<(), ()> {
        info!("Creating a cluster: {:?}", msg);

        info!("Sending to cluster");
        let request = self.raft_addr.send(admin::InitWithConfig {
            members: vec![msg.this_node.id],
        });

        RaftFut(Box::new(request.then(|res| RaftFut(match res {
            Ok(Ok(result)) => Box::new(futures::future::ok(result)),
            _ => Box::new(futures::future::err(()))
        }))))
    }
}
