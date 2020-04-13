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

pub struct Raft {
    raft_settings: raft::RaftSettings,
    http_rpc_client: rpc::HttpRpcClient,
    raft_addr: Option<actix::Addr<AppRaft>>,
    //webserver_addr: actix::Addr<rpc::WebServer>,
}

impl Raft {
    pub fn new(
        raft_settings: raft::RaftSettings,
        http_rpc_client: rpc::HttpRpcClient,
        //raft_addr: actix::Addr<AppRaft>,
        //webserver_addr: actix::Addr<rpc::WebServer>,
    ) -> Self {
        Self {
            raft_settings,
            http_rpc_client,
            raft_addr: None,
        }
    }
}

impl actix::Actor for Raft {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
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

        /*let webserver = crate::rpc::WebServer::new(
            port,
            raft_addr.clone(),
            shared_network_state.clone(),
            self.raft_settings.node_id,
        );


        let raft = raft::Raft::new(
            raft_settings,
            HttpRpcClient::new(),
        );

        raft*/

        ////////////////////// STOP /////////////////////////////

        /*let node_tracker_addr = node::NodeTracker::new().start();

        let network_addr = raft::AppNetwork::new(
            self.shared_network_state.clone(),
            self.node_id,
            node_tracker_addr,
        ).start();
        let storage = raft::AppStorage::new(self.shared_network_state.clone(), self.membership);
        let metrics = raft::AppMetrics {};

        let config = actix_raft::Config::build(String::from(match self.snapshot_dir {
            Some(ref dir) => dir,
            None => "./"
        }))
            .validate()
            .unwrap();

        let app_raft = AppRaft::new(
            self.node_id,
            config,
            network_addr.clone(),
            storage.start(),
            metrics.start().recipient(),
        );

        let port = self.rpc_port;

        let raft_addr = app_raft.start();

        let webserver = crate::rpc::WebServer::new(
            port,
            raft_addr.clone(),
            shared_network_state.clone(),
            self.node_id,
        );

        let webserver_addr = webserver.start();

        let raft = raft::Raft::new(
            raft_addr,
            webserver_addr,
            HttpRpcClient::new(),
        );*/
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        warn!("Raft is stopping");

        actix::Running::Continue
    }
}

impl actix::Handler<messages::CreateClusterRequest> for Raft {
    type Result = RaftFut<Result<(), admin::InitWithConfigError>, actix::MailboxError>; //actix::ResponseActFuture<Self, (), ()>;

    fn handle(
        &mut self,
        msg: messages::CreateClusterRequest,
        _: &mut Self::Context,
    ) -> Self::Result {
        info!("Creating a cluster: {:?}", msg);

        match self.raft_addr {
            Some(ref mut addr) => RaftFut(Box::new(addr.send(admin::InitWithConfig {
                members: vec![msg.this_node.id],
            }))),
            None => unimplemented!(), //return Box::new(ok(())
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
