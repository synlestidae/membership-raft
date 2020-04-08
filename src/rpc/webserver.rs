use actix::Addr;
use actix::prelude::Future;
use actix_raft::messages;
use bincode;
use crate::AppRaft;
use crate::node::SharedNetworkState;
use crate::raft::ClientPayload;
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
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
        data.open().read_to_end(&mut buffer).unwrap();

        let node_id = request.headers().get_one("x-node-id");
        let node_port = request.headers().get_one("x-node-port");

        match (request.remote(), node_id, node_port) {
            (Some(ip), Some(ref id_string), Some(ref port_string)) => {
                let node_id = id_string.parse::<u64>().unwrap();
                let node_port = port_string.parse::<u16>().unwrap();

                debug!("Register node {} with IP {} and port {}", node_id, ip, node_port);

                //self.shared_network.register_node(node_id, "remote-node", ip.ip(), ip.port());
            },
            result => debug!("Cannot get IP from request: {:?}", result)
        };

        info!("Handling request, waiting for the result");

        let result: Result<Vec<u8>, ()> = match request.uri().path() {
            "/client/createSessionRequest" => Ok(serialize(self.handle_create_session_request(buffer).wait().unwrap())),
            "/client/clientPayload" => Ok(serialize(self.handle_client_payload(buffer).wait().unwrap())),
            "/client/nodes" => Ok(serialize(self.handle_get_nodes())),
            "/rpc/appendEntriesRequest" => Ok(serialize(self.handle_append_entries(buffer).wait().unwrap())),
            "/rpc/voteRequest" => {
                let response: messages::VoteResponse = self.handle_vote_request(buffer).wait().unwrap().unwrap();

                Ok(serialize(response))
            },
            "/rpc/installSnapshotRequest" => Ok(serialize(self.handle_install_snapshot_request(buffer).wait().unwrap())),
            path => unimplemented!("Unknown route: {}", path)
        };

        info!("Result received");

        match result {
            Ok(bytes) => { 
                let response = Response::build()
                    .status(Status::Ok)
                    .header(ContentType::new("application", "octet-stream"))
                    .raw_header("X-Teapot-Make", "Rocket")
                    .raw_header("X-Teapot-Model", "Utopia")
                    .raw_header_adjoin("X-Teapot-Model", "Series 1")
                    .sized_body(Cursor::new(bytes))
                    .finalize();
                
                Outcome::Success(response) 
            
            },
            Err(_) => Outcome::Failure(Status::InternalServerError)
        }
    }

}

impl From<actix::MailboxError> for WebServerError {
    fn from(_: actix::MailboxError) -> Self {
        unimplemented!()
    }
}

use log::{debug, error, info};
use actix_raft::admin;

impl PostHandler {
    fn handle_client_payload(&self, data: Vec<u8>) -> WebserverFut<Result<messages::ClientPayloadResponse<crate::raft::DataResponse>, ()>, WebServerError> {
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
    }
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

        let post_client_payload_route = Route::new(Method::Post, "/client/clientPayload", post_handler.clone());
        let append_entries_route = Route::new(Method::Post, "/rpc/appendEntriesRequest", post_handler.clone());
        let vote_request_route = Route::new(Method::Post, "/rpc/voteRequest", post_handler.clone());
        let install_snapshot_route = Route::new(Method::Post, "/rpc/installSnapshotRequest", post_handler.clone());
        let create_session_request = Route::new(Method::Post, "/client/createSessionRequest", post_handler.clone());
        let get_nodes = Route::new(Method::Get, "/client/nodes", post_handler.clone());
        let admin_metrics = Route::new(Method::Post, "/admin/metrics", post_handler);

        let rocket = rocket::custom(self.rocket_config.clone())
            .mount("/", vec![
            post_client_payload_route,
            append_entries_route,
            vote_request_route,
            install_snapshot_route,
            create_session_request,
            admin_metrics,
            get_nodes
        ]);

        rocket.launch();
    }
}
