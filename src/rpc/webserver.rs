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

pub struct WebServer {
    addr: Addr<AppRaft>,
    shared_network: SharedNetworkState,
    rocket_config: Config 
}

pub struct WebServerError {
   pub error_message: String,
   pub status_code: u16
}

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

        let result: Result<Vec<u8>, _> = match request.uri().path() {
            "/client/createSessionRequest" => self.handle_create_session_request(buffer).map(serialize),
            "/client/clientPayload" => self.handle_client_payload(buffer).map(serialize),
            "/rpc/appendEntriesRequest" => self.handle_append_entries(buffer).map(serialize),
            "/rpc/voteRequest" => self.handle_vote_request(buffer).map(serialize),
            "/rpc/installSnapshotRequest" => self.handle_install_snapshot_request(buffer).map(serialize),
            path => unimplemented!("Unknown route: {}", path)
        };

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
    fn handle_client_payload(&self, data: Vec<u8>) -> Result<messages::ClientPayloadResponse<crate::raft::DataResponse>, WebServerError> {
        let payload: ClientPayload = bincode::deserialize(&data)?;

        let result = self.addr
            .send(messages::ClientPayload::new(messages::EntryNormal { data: payload.data }, messages::ResponseMode::Committed))
            .wait();

        debug!("ClientPayloadResponse: {:?}", result);

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }

    fn handle_append_entries(&self, data: Vec<u8>) -> Result<messages::AppendEntriesResponse, WebServerError> {
        debug!("Deserialising AppendEntriesRequest request of length {}", data.len());

        let payload: messages::AppendEntriesRequest<crate::raft::Transition> = bincode::deserialize(&data)?;

        debug!("Handling AppendEntriesRequest: {:?}", payload);

        let result = self.addr.send(payload).wait();

        debug!("AppendEntriesResponse: {:?}", result);

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }

    fn handle_vote_request(&self, data: Vec<u8>) -> Result<messages::VoteResponse, WebServerError> {
        debug!("Handling a vote of {} bytes", data.len());
        let payload: messages::VoteRequest = bincode::deserialize(&data)?;

        let result = self.addr.send(payload).wait();

        debug!("VoteResponse: {:?}", result);

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => {
                debug!("VoteResponse error: {:?}", err);

                Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
            },
            Err(err) => { 
                debug!("VoteResponse error: {:?}", err);

                Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
            }
        }
    }

    fn handle_install_snapshot_request(&self, data: Vec<u8>) -> Result<messages::InstallSnapshotResponse, WebServerError> {
        let payload: messages::InstallSnapshotRequest = bincode::deserialize(&data)?;

        let result = self.addr.send(payload).wait();

        debug!("InstallSnapshotResponse: {:?}", result);

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }

    fn handle_create_session_request(&self, data: Vec<u8>) -> Result<crate::rpc::CreateSessionResponse, WebServerError> {
        let mut shared_network = self.shared_network.clone();
        let request: CreateSessionRequest = bincode::deserialize(&data)?;

        let new_node = request.new_node;

        info!("New node: {:?}", new_node);

        shared_network.register_node(new_node.id, &new_node.name, new_node.address, new_node.port);

        let entry = messages::EntryNormal { 
            data: crate::raft::Transition::AddNode { 
                id: new_node.id,
                name: new_node.name,
                address: new_node.address,
                port: new_node.port 
            } 
        };

        let result = match self.addr.send(admin::ProposeConfigChange::new(vec![new_node.id], vec![])).wait()? {
            Err(admin::ProposeConfigChangeError::NodeNotLeader(Some(node_id))) => {
                let leader_node_option = shared_network.get_node(node_id);

                if let Some(leader_node) = leader_node_option {
                    Ok(CreateSessionResponse::RedirectToLeader {
                        leader_node 
                    })
                } else {
                    Ok(CreateSessionResponse::Error)
                }
            },
            Err(admin::ProposeConfigChangeError::NodeNotLeader(None)) => {
                error!("Error with creating session: NO LEADER!");

                Ok(CreateSessionResponse::Error)
            },
            Ok(_) => Ok(CreateSessionResponse::Success),
            Err(err) => { 
                error!("Error with creating session: {:?}", err);
                Ok(CreateSessionResponse::Error)
            },
        };

        let payload = messages::ClientPayload::new(entry, messages::ResponseMode::Committed);

        match self.addr.send(payload).wait()? {
            Ok(_) => result,
            Err(err) => {
                error!("Error sending ClientPayload: {:?}", err);
                return Ok(CreateSessionResponse::Error);
            }
        }
    }
}

impl WebServer {
    pub(crate) fn new(port: u16, addr: Addr<AppRaft>, shared_network_state: SharedNetworkState) -> Self {
        Self {
            addr,
            shared_network: shared_network_state,
            rocket_config: ConfigBuilder::new(Environment::Staging)
                .address("0.0.0.0")
                .port(port)
                .finalize()
                .unwrap()
        }
    }

    pub fn start(&mut self) {
        let post_handler = PostHandler {
            addr: self.addr.clone(),
            shared_network: self.shared_network.clone()
        };

        let post_client_payload_route = Route::new(Method::Post, "/client/clientPayload", post_handler.clone());
        let append_entries_route = Route::new(Method::Post, "/rpc/appendEntriesRequest", post_handler.clone());
        let vote_request_route = Route::new(Method::Post, "/rpc/voteRequest", post_handler.clone());
        let install_snapshot_route = Route::new(Method::Post, "/rpc/installSnapshotRequest", post_handler.clone());
        let create_session_request = Route::new(Method::Post, "/client/createSessionRequest", post_handler.clone());
        let admin_metrics = Route::new(Method::Post, "/admin/metrics", post_handler);

        let rocket = rocket::custom(self.rocket_config.clone())
            .mount("/", vec![
            post_client_payload_route,
            append_entries_route,
            vote_request_route,
            install_snapshot_route,
            create_session_request,
            admin_metrics 
        ]);

        rocket.launch();
    }
}
