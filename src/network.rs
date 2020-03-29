use crate::Data;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages::InstallSnapshotRequest;
use actix_raft::messages::InstallSnapshotResponse;
use actix_raft::messages::VoteRequest;
use actix_raft::messages::VoteResponse;
use actix_raft::{messages, RaftNetwork};
use std::net::IpAddr;
use crate::shared_network_state::SharedNetworkState;
use actix::fut::result;
use log::{debug, error};
use actix_raft::NodeId;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use std::io::Cursor;
use std::fmt::Debug;
use crate::config::WebserverConfig;


/// Your application's network interface actor.
pub struct AppNetwork {
    shared_network_state: SharedNetworkState,
    node_id: NodeId,
    webserver: WebserverConfig
}

impl AppNetwork {
    pub fn new(shared_network_state: SharedNetworkState, node_id: NodeId, webserver: &WebserverConfig) -> Self {
        Self {
            shared_network_state,
            node_id,
            webserver: webserver.clone()
        }
    }

    fn post<'r, S: Serialize + Debug, D: DeserializeOwned + Debug>(&mut self, node_id: NodeId, msg: S, path: &str) -> Result<D, reqwest::Error> {//ResponseActFuture<A, R, reqwest::Error> {
        let node_option = self.shared_network_state.get_node(node_id);

        match node_option {
            Some(node) => {
                let uri = format!("http://{}:{}{}", node.address, node.port, path);
                debug!("Sending msg to node {} at {}", node_id, uri); 
                debug!("Serializing: {:?}", msg);

                let body_bytes: Vec<u8>  = bincode::serialize(&msg).unwrap();
                let body: Body = body_bytes.into();

                let responseResult = reqwest::blocking::Client::new().post(&uri)
                    .header("User-Agent", "Membership-Raft")
                    .header("X-Node-Id", self.node_id)
                    .header("X-Node-Port", self.webserver.port)
                    .body(body)
                    .send();

                let res = match responseResult {
                    Ok(resp) => {
                        let bytes = resp.bytes().unwrap().into_iter().collect::<Vec<u8>>();
                        debug!("Deserializing {} bytes from {}", bytes.len(), path);

                        let response: D = bincode::deserialize_from(Cursor::new(bytes)).unwrap();
                        debug!("Deserialized: {:?}", response);

                        Ok(response)
                    },
                    Err(err) => {
                        error!("Error in response: {:?}", err);

                        Err(err)
                    }
                };

                return res;
            },
            None => {
                error!("Failed to get node with id {}", node_id);
                unimplemented!()
            }
        }
    }

    fn handle_http<S: Serialize + Debug, D: 'static + DeserializeOwned + Debug>(&mut self, node_id: NodeId, path: &str, msg: S) -> ResponseActFuture<Self, D, ()> {
        let node_option = self.shared_network_state.get_node(node_id);

        match node_option {
            Some(node) => {
                return match self.post(node.id, msg, path) {
                    Ok(resp) => Box::new(result(Ok(resp))),
                    Err(err) => { 
                        error!("Error making request: {:?}", err);
                        Box::new(result(Err(())))
                    }
                };
            },
            None => { 
                error!("Unable to find node with id: {}", node_id );
                return Box::new(result(Err(()))) 
            }
        }
    }
}

impl Actor for AppNetwork {
    type Context = Context<Self>;

    // ... snip ... other actix methods can be implemented here as needed.
}

// Ensure you impl this over your application's data type. Here, it is `Data`.
impl RaftNetwork<Data> for AppNetwork {}

// Then you just implement the various message handlers.
// See the network chapter for details.
impl Handler<messages::AppendEntriesRequest<Data>> for AppNetwork {
    type Result = ResponseActFuture<Self, messages::AppendEntriesResponse, ()>;

    fn handle(
        &mut self,
        msg: messages::AppendEntriesRequest<Data>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let node_option = self.shared_network_state.get_node(msg.target);

        match node_option {
            Some(node) => {
                return match self.post(node.id, msg, "/rpc/appendEntriesRequest") {
                    Ok(resp) => Box::new(result(Ok(resp))),
                    Err(_) => Box::new(result(Err(())))
                };

            },
            None => return Box::new(result(Err(())))
        }
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = ResponseActFuture<Self, InstallSnapshotResponse, ()>;

    fn handle(&mut self, msg: InstallSnapshotRequest, _ctx: &mut Self::Context) -> Self::Result {
        self.handle_http(msg.target, "/rpc/installSnapshotRequest", msg)
    }
}

use futures_util::FutureExt;
use futures_util::TryFutureExt;
use actix::fut::IntoActorFuture;
use reqwest::blocking::Body;

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<VoteRequest> for AppNetwork {
    type Result = ResponseActFuture<Self, VoteResponse, ()>;

    fn handle(&mut self, msg: VoteRequest, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling VoteRequest: {:?}", msg);

        self.handle_http(msg.target, "/rpc/voteRequest", msg)
    }
}


