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

/// Your application's network interface actor.
pub struct AppNetwork {
    shared_network_state: SharedNetworkState,
    node_id: NodeId,
    webserver: WebserverConfig,
    node_event_sender: Sender<NodeEvent>,
    http_helper: HttpHelper,
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
            http_helper: HttpHelper::new(),
        }
    }

    fn post<S: Serialize + Debug, D: 'static + DeserializeOwned + Debug>(
        &mut self,
        node_id: NodeId,
        msg: S,
        path: &str,
    ) -> HttpFuture<D, ()> {
        //ResponseActFuture<A, R, reqwest::Error> {
        let node_option = self.shared_network_state.get_node(node_id);

        match node_option {
            Some(node) => {
                let uri =
                    Url::parse(&format!("http://{}:{}{}", node.host, node.port, path)).unwrap();

                debug!("Sending msg to node {} at {}", node_id, uri);
                debug!("Serializing: {:?}", msg);

                self.http_helper.post_to_uri(uri, msg)

                /*let body_bytes: Vec<u8>  = bincode::serialize(&msg).unwrap();
                let body: Body = body_bytes.into();

                let response_result = reqwest::blocking::Client::new().post(&uri)
                    .header("User-Agent", "Membership-Raft")
                    .header("X-Node-Id", self.node_id)
                    .header("X-Node-Port", self.webserver.port)
                    .body(body)
                    .send();

                let res = match response_result {
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
                };*/
                //return res;
            }
            None => {
                error!("Failed to get node with id {}", node_id);
                unimplemented!()
            }
        }
    }

    fn handle_http<S: Serialize + Debug, D: 'static + DeserializeOwned + Debug>(
        &mut self,
        node_id: NodeId,
        path: &str,
        msg: S,
    ) -> AppNetworkFut<D, ()> {
        let node_option = self.shared_network_state.get_node(node_id);

        match node_option {
            Some(node) => Box::new(self.post(node.id, msg, path)),
            None => {
                error!("Unable to find node with id: {}", node_id);
                //panic!("Not gonna happen");
                Box::new(err(()))
            }
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

        info!("AppendEntriesRequest: {:?}", msg);

        let node_event_sender = self.node_event_sender.clone();
        let node_event_sender2 = self.node_event_sender.clone();

        match node_option {
            Some(node) => {
                let node2 = node.clone();
                return Box::new(
                    self.post(node.id, msg, "/rpc/appendEntriesRequest")
                        .map(move |resp| {
                            info!("AppendEntriesRequest response: {:?}", resp);

                            drop(node_event_sender.send(NodeEvent::ok(&node)));

                            resp
                        })
                        .map_err(move |_| {
                            drop(node_event_sender2.send(NodeEvent::err(&node2)));

                            ()
                        }),
                );
            }
            None => panic!("Hang on, where do I send this thing?"),
        }
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = AppNetworkFut<InstallSnapshotResponse, ()>;

    fn handle(&mut self, msg: InstallSnapshotRequest, _ctx: &mut Self::Context) -> Self::Result {
        self.handle_http(msg.target, "/rpc/installSnapshotRequest", msg)
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<VoteRequest> for AppNetwork {
    type Result = AppNetworkFut<VoteResponse, ()>;

    fn handle(&mut self, msg: VoteRequest, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling VoteRequest: {:?}", msg);

        self.handle_http::<VoteRequest, VoteResponse>(msg.target, "/rpc/voteRequest", msg)
    }
}
