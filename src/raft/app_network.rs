use crate::config::WebserverConfig;
use crate::http_helper::{HttpFuture, HttpHelper};
use crate::node::NodeEvent;
use crate::node::SharedNetworkState;
use crate::raft::Transition;
use actix::fut::result;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages::InstallSnapshotRequest;
use actix_raft::messages::InstallSnapshotResponse;
use actix_raft::messages::VoteRequest;
use actix_raft::messages::VoteResponse;
use actix_raft::NodeId;
use actix_raft::{messages, RaftNetwork};
use futures::future::err;
use futures::Future;
use log::{debug, error, info};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::mpsc::Sender;
use crate::rpc::HttpRpcClient;
use crate::rpc::RpcClient;

/// Your application's network interface actor.
pub struct AppNetwork {
    shared_network_state: SharedNetworkState,
    node_id: NodeId,
    webserver: WebserverConfig,
    node_event_sender: Sender<NodeEvent>,
    rpc_client: HttpRpcClient
}

impl AppNetwork {
    pub fn new(
        shared_network_state: SharedNetworkState,
        node_id: NodeId,
        webserver: &WebserverConfig,
        node_event_sender: Sender<NodeEvent>,
    ) -> Self {
        Self {
            shared_network_state,
            node_id,
            webserver: webserver.clone(),
            node_event_sender,
            rpc_client: HttpRpcClient::new()
        }
    }
}

impl Actor for AppNetwork {
    type Context = Context<Self>;

    // ... snip ... other actix methods can be implemented here as needed.
}

// Ensure you impl this over your application's data type. Here, it is `Data`.
impl RaftNetwork<Transition> for AppNetwork {}

type AppNetworkFut<E, R> = Box<dyn Future<Item = E, Error = R>>;

// Then you just implement the various message handlers.
// See the network chapter for details.
impl Handler<messages::AppendEntriesRequest<Transition>> for AppNetwork {
    type Result = AppNetworkFut<messages::AppendEntriesResponse, ()>;

    fn handle(
        &mut self,
        msg: messages::AppendEntriesRequest<Transition>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let node_option = self.shared_network_state.get_node(msg.target);

        if let Some(node) = node_option {
            let node_ok = NodeEvent::ok(&node);
            let node_err = NodeEvent::err(&node);

            let sender1 = self.node_event_sender.clone();
            let sender2 = self.node_event_sender.clone();

            Box::new(self.rpc_client
                .append_entries(&node.rpc_url(), msg)
                .map(move |resp| { 
                    drop(sender1.send(node_ok));

                    resp
                })
                .map_err(move |_| {
                    drop(sender2.send(node_err));
                
                    ()
                }))
        } else {
            panic!("Unsure where to send this")
        }
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = AppNetworkFut<InstallSnapshotResponse, ()>;

    fn handle(&mut self, msg: InstallSnapshotRequest, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling InstallSnapshotRequest: {:?}", msg);

        let node = self.shared_network_state.get_node(msg.target).expect("Unable to find node");

        Box::new(self.rpc_client.install_snapshot(&node.rpc_url(), msg).map_err(|_| ()))
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<VoteRequest> for AppNetwork {
    type Result = AppNetworkFut<VoteResponse, ()>;

    fn handle(&mut self, msg: VoteRequest, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling VoteRequest: {:?}", msg);

        let node = self.shared_network_state.get_node(msg.target).expect("Unable to find node");

        Box::new(self.rpc_client.vote(&node.rpc_url(), msg).map_err(|_| ()))
    }
}
