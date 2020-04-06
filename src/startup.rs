use crate::rpc;
use crate::AppRaft;
use actix_raft::admin;
use actix;
use log::*;
use actix::fut::result;
use crate::node;
use actix_raft::NodeId;
use crate::rpc::CreateSessionResponse;

pub struct StartupActor {
    pub node_id: actix_raft::NodeId,
    pub raft_addr: actix::Addr<AppRaft>,
    pub admin_api: rpc::AdminNetwork 
}

impl StartupActor {
    pub fn new(
        node_id: actix_raft::NodeId,
        raft_addr: actix::Addr<AppRaft>,
        admin_api: rpc::AdminNetwork) -> Self {
        Self {
            raft_addr,
            node_id,
            admin_api
        }
    }
}

impl actix::Actor for StartupActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {

    }
}

#[derive(Debug)]
pub struct ClusterConfig {
}

#[derive(Debug)]
pub enum StartupRequest {
    NewCluster { cluster_config: ClusterConfig, config: crate::config::Config },
    ExistingCluster { config: crate::config::Config } 
}

impl actix::Message for StartupRequest {
    type Result = Result<StartupResponse, StartupErr>;
}

#[derive(Debug)]
pub struct StartupResponse {
    leader_node_id: NodeId
}

#[derive(Debug)]
pub struct StartupErr {
}

use actix::fut::ActorFuture;

/*struct StartupFuture {
}

impl actix::ActorFuture for StartupFuture {
    type Item = StartupResponse;
    type Error = StartupErr;
    type Actor = StartupActor;

    fn poll(&mut self, srv: &mut Self::Actor, ctx: &mut <Self::Actor as actix::Actor>::Context) -> std::result::Result<utures::Async<StartupResponse>, StartupErr> {
        todo!()
    }
}*/

use actix::prelude::Future;

impl actix::Handler<StartupRequest> for StartupActor {
    type Result = actix::prelude::ResponseFuture<StartupResponse, StartupErr>;

    fn handle(&mut self, msg: StartupRequest, _: &mut <Self as actix::Actor>::Context) -> Self::Result { 
        match msg {
            StartupRequest::NewCluster { cluster_config: _, config: _ } => {
                info!("Starting a new cluster");

                let init_with_config = admin::InitWithConfig {
                    members: vec![self.node_id]
                };

                info!("Initialising with just one member (me!)");

                //let future: Box<dyn ActorFuture<Item=StartupResponse, Error=StartupErr, Actor = Self>> = Box::new(self.raft_addr.send(init_with_config));
                Box::new(self.raft_addr.send(init_with_config).map(|r| {
                    info!("Successfully added config: {:?}", r);

                    StartupResponse { leader_node_id: 0 } //TODO
                }).map_err(|err| {
                    error!("Error adding config: {:?}", err);

                    StartupErr {}
                }))
            },
            StartupRequest::ExistingCluster { config } => {
                let msg = rpc::CreateSessionRequest {
                    new_node: node::AppNode {
                        id: self.node_id,
                        name: config.name.to_string(),
                        host: config.webserver.host.clone(),
                        port: config.webserver.port
                    }
                };
                let url = reqwest::Url::parse(&format!("http://{}/client/createSessionRequest", config.bootstrap_hosts[0])).unwrap();
                Box::new(self.admin_api.session_request(url, msg)
                    .map(|r| StartupResponse { leader_node_id: match r {
                        CreateSessionResponse::Success { leader_node_id } => leader_node_id,
                        _ => unimplemented!(),
                    }})
                    .map_err(|_| StartupErr { }))
            }
        }
    }
}
