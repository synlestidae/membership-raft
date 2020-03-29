use rocket_contrib::json::Json;
use rocket::response;
use rocket::response::Responder;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use rocket::response::status::Custom;
use rocket::config::{Config, ConfigBuilder, Environment};
use rocket::request::FromRequest;
use rocket::handler::Outcome;
use rocket::Request;
use rocket::http::Status;
use rocket::http::Cookies;
use rocket::http::Cookie;
use crate::AppRaft;
use actix::Addr;
use crate::client_payload::ClientPayload;
use rocket::data::DataStream;
use rocket::Data as RocketData;
use std::io::Read;
use std::marker::PhantomData;
use rocket::Handler;
use bincode;
use std::convert::From;
use actix_raft::messages;

use rocket::Route;
use rocket::http::Method;
use rocket::Response;
use rocket::http::ContentType;

use crate::Data;

use crate::shared_network_state::SharedNetworkState;

use rocket::Rocket;
//use futures_util::FuturesExt;
use actix::prelude::Future;
use rocket;

pub struct WebServer {
    port: u16,
    addr: Addr<AppRaft>,
    shared_network: SharedNetworkState,
    rocket_config: Config 
}

pub struct WebServerError {
   error_message: String,
   status_code: u16
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

pub type ResultResponse<T> = Result<Custom<Json<T>>, WebServerError>;

#[derive(Clone)]
struct PostHandler {
    addr: Addr<AppRaft>,
}

fn serialize<S: Serialize>(s: S) -> Vec<u8> {
    bincode::serialize(&s).unwrap()
}

use std::io::Cursor;

impl Handler for PostHandler {
    fn handle<'r>(&self, request: &'r Request, data: RocketData) -> Outcome<'r> {
        let mut buffer = Vec::new();
        data.open().read_to_end(&mut buffer);

        let result: Result<Vec<u8>, _> = match request.uri().path() {
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
            Err(error) => Outcome::Failure(Status::InternalServerError)
        }
    }
}

impl From<actix::MailboxError> for WebServerError {
    fn from(err: actix::MailboxError) -> Self {
        unimplemented!()
    }
}

use log::{debug, error};

impl PostHandler {
    fn handle_client_payload(&self, data: Vec<u8>) -> Result<messages::ClientPayloadResponse<crate::DataResponse>, WebServerError> {
        let payload: ClientPayload = bincode::deserialize(&data)?;

        let result = self.addr
            .send(messages::ClientPayload::new(messages::EntryNormal { data: payload.data }, messages::ResponseMode::Committed))
            .wait();

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }

    fn handle_append_entries(&self, data: Vec<u8>) -> Result<messages::AppendEntriesResponse, WebServerError> {
        debug!("Deserialising AppendEntriesRequest request of length {}", data.len());

        let payload: messages::AppendEntriesRequest<Data> = bincode::deserialize(&data)?;

        debug!("Handling AppendEntriesRequest: {:?}", payload);

        let result = self.addr.send(payload).wait();

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }

    fn handle_vote_request(&self, data: Vec<u8>) -> Result<messages::VoteResponse, WebServerError> {
        let payload: messages::VoteRequest = bincode::deserialize(&data)?;

        let result = self.addr.send(payload).wait();

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }

    fn handle_install_snapshot_request(&self, data: Vec<u8>) -> Result<messages::InstallSnapshotResponse, WebServerError> {
        let payload: messages::InstallSnapshotRequest = bincode::deserialize(&data)?;

        let result = self.addr.send(payload).wait();

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 }),
            Err(err) => Err(WebServerError { error_message: format!("Error handling client payload: {:?}", err), status_code: 500 })
        }
    }
}

impl WebServer {
    pub(crate) fn new(port: u16, addr: Addr<AppRaft>) -> Self {
        Self {
            port,
            addr,
            shared_network: SharedNetworkState::new(),
            rocket_config: ConfigBuilder::new(Environment::Staging)
                .address("0.0.0.0")
                .port(port)
                .finalize()
                .unwrap()
        }
    }

    pub fn start(&mut self) {
        let post_handler = PostHandler {
            addr: self.addr.clone()
        };

        let post_client_payload_route = Route::new(Method::Post, "/client/clientPayload", post_handler.clone());
        let append_entries_route = Route::new(Method::Post, "/rpc/appendEntriesRequest", post_handler.clone());
        let vote_request_route = Route::new(Method::Post, "/rpc/voteRequest", post_handler.clone());
        let install_snapshot_route = Route::new(Method::Post, "/rpc/installSnapshotRequest", post_handler);

        let rocket = rocket::custom(self.rocket_config.clone()).mount("/", vec![
            post_client_payload_route,
            append_entries_route,
            vote_request_route,
            install_snapshot_route 
        ]);

        rocket.launch();
    }
}
