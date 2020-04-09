use actix::Addr;
use actix::prelude::Future;
use actix_raft::messages;
use bincode;
use crate::AppRaft;
use crate::node::SharedNetworkState;
use crate::raft::ClientPayload;
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use crate::rpc::RpcRequest;
use rocket::Data as RocketData;
use rocket::Handler;
use rocket::Request;
use rocket::Response;
use rocket::Route;
use rocket::config::{Config, ConfigBuilder, Environment};
use rocket::handler::Outcome;
use rocket::http::ContentType;
use rocket::http::Method;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::response;
use rocket;
use serde::{Serialize};
use std::convert::From;
use std::io::Read;
use actix_raft::NodeId;
use futures::future::{ok, err};
use serde_json::Value;
use serde_json::{from_slice};

pub struct WebServer {
    addr: Addr<AppRaft>,
    shared_network: SharedNetworkState,
    rocket_config: Config,
    node_id: NodeId
}

#[derive(Debug)]
pub struct WebServerError {
   pub error_message: String,
   pub status_code: u16
}

pub type WebserverFut<E, R> = Box<dyn Future<Item=E, Error=R>>;

impl From<Box<bincode::ErrorKind>> for WebServerError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Self {
            error_message: format!("Error deserialising data: {}", error),
            status_code: 400
        }
    }
}

impl<'r> Responder<'r> for WebServerError {
    fn respond_to(self, _request: &Request) -> response::Result<'r> {
        Err(Status::InternalServerError)
    }
}

#[derive(Clone)]
struct PostHandler {
    node_id: NodeId,
    addr: Addr<AppRaft>,
    shared_network: SharedNetworkState,
}

fn serialize<S: Serialize>(s: S) -> Vec<u8> {
    bincode::serialize(&s).unwrap()
}

use std::io::Cursor;

impl Handler for PostHandler {
    fn handle<'r>(&self, request: &'r Request, data: RocketData) -> Outcome<'r> {
        let mut buffer = Vec::new();

        match data.open().read_to_end(&mut buffer) {
            Ok(_) => {},
            Err(err) => return Outcome::failure(Status::InternalServerError)
        }

        let request = match serde_json::from_slice(&buffer) {
            Ok(o) => o,
            Err(err) => return Outcome::failure(Status::BadRequest),
        };

        let value = match self.handle_request(request) {
            Ok(v) => v,
            Err(err) => return Outcome::failure(Status::InternalServerError)
        };

        let value_bytes = match serde_json::to_vec(&value) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("Error serializing response: {:?}", err);

                return Outcome::failure(Status::InternalServerError);
            }
        };

        let response = Response::build()
            .status(Status::Ok)
            .header(ContentType::new("application", "json"))
            .raw_header("X-Node-Id", self.node_id.to_string())
            .raw_header("Server", "Membership")
            .sized_body(Cursor::new(value_bytes))
            .finalize();
        
        Outcome::Success(response) 
    }

}

impl From<actix::MailboxError> for WebServerError {
    fn from(_: actix::MailboxError) -> Self {
        unimplemented!()
    }
}

use log::{debug, error, info};
use actix_raft::admin;
use actix_raft;

impl PostHandler {
    fn handle_request<'r>(&self, object: RpcRequest) -> Result<serde_json::Value, WebServerError> {
        let value: serde_json::Value = match object {
            RpcRequest::JoinCluster(request) => {
                let new_node = request.new_node;

                info!("New node: {:?}", new_node);

                self.shared_network.register_node(new_node.id, &new_node.name, new_node.host.clone(), new_node.port);

                let entry = messages::EntryNormal { 
                    data: crate::raft::Transition::AddNode { 
                        id: new_node.id,
                        name: new_node.name,
                        host: new_node.host,
                        port: new_node.port 
                    } 
                };
                let payload = messages::ClientPayload::new(entry, messages::ResponseMode::Applied);
                self.addr.send(payload).wait().unwrap();

                let proposal_result = self.addr
                    .send(admin::ProposeConfigChange::new(vec![new_node.id], vec![]))
                    .wait()
                    .unwrap();
                
                match serde_json::to_value(match proposal_result {
                    Ok(()) => CreateSessionResponse::Success { leader_node_id: self.node_id },
                    Err(actix_raft::admin::ProposeConfigChangeError::Noop) => CreateSessionResponse::Success { leader_node_id: self.node_id },
                    Err(actix_raft::admin::ProposeConfigChangeError::NodeNotLeader(None)) => CreateSessionResponse::Error,
                    Err(actix_raft::admin::ProposeConfigChangeError::NodeNotLeader(Some(leader_node_id))) => {
                        if let Some(leader_node) = self.shared_network.get_node(leader_node_id) {
                            CreateSessionResponse::RedirectToLeader { leader_node }
                        } else {
                            CreateSessionResponse::Error
                        }
                    },
                    Err(actix_raft::admin::ProposeConfigChangeError::Internal) => return Err(
                        WebServerError { 
                            error_message: format!("Error handling client payload"), 
                            status_code: 500 
                        }
                    ),
                    Err(e) => {
                        error!("Error with creating session: {:?}", e);

                        CreateSessionResponse::Error
                    },
                }) {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Error in converting value to JSON: {:?}", err);
                        
                        return Err(WebServerError { 
                            error_message: format!("Error handling client payload"), 
                            status_code: 500 
                        });
                    }
                }
            },
            RpcRequest::GetNodes => unimplemented!(),
            RpcRequest::AppendEntries(append_entries_request) => unimplemented!(),
            RpcRequest::Vote(vote_request) => unimplemented!(),
            RpcRequest::InstallSnapshot(install_snapshot_request) => unimplemented!()
        };

        Ok(value)

        /*let value_bytes = match serde_json::to_vec(&value) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("Error serializing response: {:?}", err);

                return Outcome::failure(Status::InternalServerError);
            }
        };*/
    }
    /*fn handle_client_payload(&self, data: Vec<u8>) -> WebserverFut<Value, ()> {
        let payload: ClientPayload = bincode::deserialize(&data).unwrap();

        Box::new(self.addr
            .send(messages::ClientPayload::new(messages::EntryNormal { data: payload.data }, messages::ResponseMode::Committed))
            .map(|result| result.map_err(|_| ()))
            .map_err(|_| WebServerError { error_message: format!("Error handling client payload"), status_code: 500 } ))

    }

    fn handle_append_entries(&self, data: Vec<u8>) -> WebserverFut<Result<messages::AppendEntriesResponse, ()>, WebServerError> {
        debug!("Deserialising AppendEntriesRequest request of length {}", data.len());

        let payload: messages::AppendEntriesRequest<crate::raft::Transition> = bincode::deserialize(&data).unwrap();

        debug!("Handling AppendEntriesRequest: {:?}", payload);

        Box::new(self.addr.send(payload)
            .map_err(|_| WebServerError { error_message: format!("Error handling client payload"), status_code: 500 }))
    }

    fn handle_vote_request(&self, data: Vec<u8>) -> WebserverFut<Result<messages::VoteResponse, ()>, WebServerError> {
        debug!("Handling a vote of {} bytes", data.len());
        let payload: messages::VoteRequest = bincode::deserialize(&data).unwrap();

        debug!("VoteRequest: {:?}", payload);

        Box::new(self.addr.send(payload)
            .map_err(|_| WebServerError { error_message: format!("Error handling client payload"), status_code: 500 })
        )
    }

    fn handle_install_snapshot_request(&self, data: Vec<u8>) -> WebserverFut<Result<messages::InstallSnapshotResponse, ()>, WebServerError> {
        let payload: messages::InstallSnapshotRequest = bincode::deserialize(&data).unwrap();

        debug!("InstallSnapshotResponse: {:?}", payload);

        Box::new(self.addr.send(payload)
            .map_err(|_| WebServerError { error_message: format!("Error handling client payload"), status_code: 500 })
        )
    }

    fn handle_get_nodes(&self) -> Vec<crate::node::AppNode> {
        debug!("Getting nodes");
        self.shared_network.get_nodes()
    }

    fn handle_create_session_request(&self, data: Vec<u8>) -> WebserverFut<crate::rpc::CreateSessionResponse, WebServerError> {
        let mut shared_network = self.shared_network.clone();
        let request: CreateSessionRequest = bincode::deserialize(&data).unwrap();

        let new_node = request.new_node;

        info!("New node: {:?}", new_node);

        shared_network.register_node(new_node.id, &new_node.name, new_node.host.clone(), new_node.port);

        let entry = messages::EntryNormal { 
            data: crate::raft::Transition::AddNode { 
                id: new_node.id,
                name: new_node.name,
                host: new_node.host,
                port: new_node.port 
            } 
        };

        let payload = messages::ClientPayload::new(entry, messages::ResponseMode::Applied);

        let client_future = self.addr.send(payload)
            .map(|r| r)
            .map_err(|err| {
                error!("Error sending ClientPayload: {:?}", err);

                unimplemented!("Error sending ClientPayload: {:?}", err)
            });

        let node_id = self.node_id;

        Box::new(self.addr.send(admin::ProposeConfigChange::new(vec![new_node.id], vec![])) 
            .map(move |result| {
                match result {
                    Ok(_) => CreateSessionResponse::Success { leader_node_id: node_id },
                    Err(admin::ProposeConfigChangeError::NodeNotLeader(Some(node_id))) => {
                        let leader_node_option = shared_network.get_node(node_id);

                        if let Some(leader_node) = leader_node_option {
                            error!("Redirecting client to leader");
                            CreateSessionResponse::RedirectToLeader {
                                leader_node 
                            }
                        } else {
                            error!("Cannot find info for leader node");
                            CreateSessionResponse::Error
                        }
                    },
                    Err(admin::ProposeConfigChangeError::NodeNotLeader(None)) => {
                        error!("Error with creating session: NO LEADER!");

                        CreateSessionResponse::Error
                    },
                    Err(e) => {
                        error!("Error with creating session: {:?}", e);

                        CreateSessionResponse::Error
                    },
                }
            })
            .then(|res| match res {
                Err(e) => unimplemented!(),
                Ok(CreateSessionResponse::Success { leader_node_id }) => {
                    return Box::new(client_future
                        .map(move |_| CreateSessionResponse::Success { leader_node_id })
                        .map_err(|_| unimplemented!()))
                },
                Ok(e) => unimplemented!()
            }))
    }*/
}

impl WebServer {
    pub(crate) fn new(port: u16, addr: Addr<AppRaft>, shared_network_state: SharedNetworkState, node_id: NodeId) -> Self {
        Self {
            addr,
            shared_network: shared_network_state,
            node_id,
            rocket_config: ConfigBuilder::new(Environment::Staging)
                .address("0.0.0.0")
                .port(port)
                .finalize()
                .unwrap()
        }
    }

    pub fn start(&mut self, node_id: NodeId) {
        let post_handler = PostHandler {
            node_id,
            addr: self.addr.clone(),
            shared_network: self.shared_network.clone()
        };

        let route = Route::new(Method::Post, "/rpc", post_handler);

        let rocket = rocket::custom(self.rocket_config.clone())
            .mount("/", vec![route]);

        rocket.launch();
    }
}
