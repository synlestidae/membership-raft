use crate::node::SharedNetworkState;
use crate::rpc::CreateSessionResponse;
use crate::rpc::RpcRequest;
use crate::AppRaft;
use actix::prelude::Future;
use actix::Addr;
use actix_raft::messages;
use actix_raft::NodeId;
use bincode;
use rocket;
use rocket::config::{Config, ConfigBuilder, Environment};
use rocket::handler::Outcome;
use rocket::http::ContentType;
use rocket::http::Method;
use rocket::http::Status;
use rocket::response;
use rocket::response::Responder;
use rocket::Data as RocketData;
use rocket::Handler;
use rocket::Request;
use rocket::Response;
use rocket::Route;
use std::convert::From;
use std::io::Read;

pub struct WebServer {
    addr: Addr<AppRaft>,
    shared_network: SharedNetworkState,
    rocket_config: Config,
    node_id: NodeId,
}

#[derive(Debug)]
pub struct WebServerError {
    pub error_message: String,
    pub status_code: u16,
}

pub type WebserverFut<E, R> = Box<dyn Future<Item = E, Error = R>>;

impl From<()> for WebServerError {
    fn from(_: ()) -> Self {
        Self {
            error_message: format!("Unknown error"),
            status_code: 500,
        }
    }
}

impl From<Box<bincode::ErrorKind>> for WebServerError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Self {
            error_message: format!("Error deserialising data: {}", error),
            status_code: 500,
        }
    }
}

impl
    From<
        actix_raft::messages::ClientError<
            crate::raft::Transition,
            crate::raft::DataResponse,
            crate::error::Error,
        >,
    > for WebServerError
{
    fn from(
        error: actix_raft::messages::ClientError<
            crate::raft::Transition,
            crate::raft::DataResponse,
            crate::error::Error,
        >,
    ) -> Self {
        Self {
            error_message: format!("Error : {}", error),
            status_code: 500,
        }
    }
}

impl From<serde_json::Error> for WebServerError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            error_message: format!("Error deserialising data: {}", error),
            status_code: 400,
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

use std::io::Cursor;

impl Handler for PostHandler {
    fn handle<'r>(&self, request: &'r Request, data: RocketData) -> Outcome<'r> {
        let mut buffer = Vec::new();

        // TODO move this code into a Result method so I can use ?

        match data.open().read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(err) => {
                error!("Error reading request into buffer: {:?}", err);

                return Outcome::failure(Status::InternalServerError);
            }
        }

        let request = match serde_json::from_slice(&buffer) {
            Ok(o) => o,
            Err(err) => {
                error!("Error parsing request JSON: {:?}", err);

                return Outcome::failure(Status::BadRequest);
            }
        };

        let value = match self.handle_request(request) {
            Ok(v) => v,
            Err(err) => {
                error!("Error handling request: {:?}", err);

                return Outcome::failure(Status::InternalServerError);
            }
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
    fn from(err: actix::MailboxError) -> Self {
        WebServerError {
            error_message: format!("Mailbox error: {:?}", err),
            status_code: 500,
        }
    }
}

use actix_raft;
use actix_raft::admin;
use log::{debug, error, info};

impl PostHandler {
    fn handle_request<'r>(&self, object: RpcRequest) -> Result<serde_json::Value, WebServerError> {
        let value: serde_json::Value = match object {
            RpcRequest::JoinCluster(request) => {
                let new_node = request.new_node;

                info!("New node: {:?}", new_node);

                self.shared_network.register_node(
                    new_node.id,
                    &new_node.name,
                    new_node.host.clone(),
                    new_node.port,
                );

                let entry = messages::EntryNormal {
                    data: crate::raft::Transition::AddNode {
                        id: new_node.id,
                        name: new_node.name,
                        host: new_node.host,
                        port: new_node.port,
                    },
                };
                let payload = messages::ClientPayload::new(entry, messages::ResponseMode::Applied);
                self.addr.send(payload).wait()??;

                let proposal_result = self
                    .addr
                    .send(admin::ProposeConfigChange::new(vec![new_node.id], vec![]))
                    .wait()
                    .unwrap();

                serde_json::to_value(match proposal_result {
                    Ok(()) => CreateSessionResponse::Success {
                        leader_node_id: self.node_id,
                    },
                    Err(actix_raft::admin::ProposeConfigChangeError::Noop) => {
                        CreateSessionResponse::Success {
                            leader_node_id: self.node_id,
                        }
                    }
                    Err(actix_raft::admin::ProposeConfigChangeError::NodeNotLeader(None)) => {
                        CreateSessionResponse::Error
                    }
                    Err(actix_raft::admin::ProposeConfigChangeError::NodeNotLeader(Some(
                        leader_node_id,
                    ))) => {
                        if let Some(leader_node) = self.shared_network.get_node(leader_node_id) {
                            CreateSessionResponse::RedirectToLeader { leader_node }
                        } else {
                            CreateSessionResponse::Error
                        }
                    }
                    Err(actix_raft::admin::ProposeConfigChangeError::Internal) => {
                        return Err(WebServerError {
                            error_message: format!("Error handling client payload"),
                            status_code: 500,
                        })
                    }
                    Err(e) => {
                        error!("Error with creating session: {:?}", e);

                        CreateSessionResponse::Error
                    }
                })?
            }
            RpcRequest::GetNodes => serde_json::to_value(&self.shared_network.get_nodes())?,
            RpcRequest::AppendEntries(append_entries_request) => {
                let value: messages::AppendEntriesResponse =
                    self.addr.send(append_entries_request).wait()??;
                serde_json::to_value(value)?
            }
            RpcRequest::Vote(vote_request) => {
                let value: messages::VoteResponse = self.addr.send(vote_request).wait()??;
                serde_json::to_value(value)?
            }
            RpcRequest::InstallSnapshot(install_snapshot_request) => {
                let value: messages::InstallSnapshotResponse =
                    self.addr.send(install_snapshot_request).wait()??;
                serde_json::to_value(value)?
            }
        };

        Ok(value)
    }
}

impl WebServer {
    pub(crate) fn new(
        port: u16,
        addr: Addr<AppRaft>,
        shared_network_state: SharedNetworkState,
        node_id: NodeId,
    ) -> Self {
        Self {
            addr,
            shared_network: shared_network_state,
            node_id,
            rocket_config: ConfigBuilder::new(Environment::Staging)
                .address("0.0.0.0")
                .port(port)
                .finalize()
                .unwrap(),
        }
    }

    pub fn start(&mut self, node_id: NodeId) {
        let post_handler = PostHandler {
            node_id,
            addr: self.addr.clone(),
            shared_network: self.shared_network.clone(),
        };

        let route = Route::new(Method::Post, "/rpc", post_handler);

        let rocket = rocket::custom(self.rocket_config.clone()).mount("/", vec![route]);

        rocket.launch();
    }
}
