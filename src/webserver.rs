use rocket_contrib::json::Json;
use rocket::response;
use rocket::response::Responder;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use rocket::response::status::Custom;
use rocket::config::{Config, Environment};
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

use crate::Data;

use crate::shared_network_state::SharedNetworkState;

use rocket::Rocket;


pub struct WebServer {
    addr: Addr<AppRaft>,
    shared_network: SharedNetworkState,
    rocket: Rocket
}

struct WebServerError {
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

type ResultResponse<T> = Result<Custom<Json<T>>, WebServerError>;

#[derive(Clone)]
struct PostHandler {
    addr: Addr<AppRaft>,
}

impl Handler for PostHandler {
    fn handle<'r>(&self, request: &'r Request, data: RocketData) -> Outcome<'r> {
        let mut buffer = Vec::new();
        data.open().read_to_end(&mut buffer);

        let result = match request.uri().path() {
            "/client/clientPayload" => self.handle_client_payload(buffer),
            "/rpc/appendEntriesRequest" => self.handle_append_entries(buffer),
            "/rpc/voteRequest" => self.handle_vote_request(buffer),
            "/rpc/installSnapshotRequest" => self.handle_install_snapshot_request(buffer),
            path => unimplemented!("Unknown route: {}", path)
        };

        match result {
            Ok(_) => Outcome::Success(Response::new()),
            Err(error) => Outcome::Failure(Status::InternalServerError)
        }
    }
}

impl PostHandler {
    fn handle_client_payload(&self, data: Vec<u8>) -> Result<(), WebServerError> {
        let payload: ClientPayload = bincode::deserialize(&data)?;

        self.addr.do_send(messages::ClientPayload::new(messages::EntryNormal { data: payload.data }, messages::ResponseMode::Committed));

        Ok(())
    }

    fn handle_append_entries(&self, data: Vec<u8>) -> Result<(), WebServerError> {
        let payload: messages::AppendEntriesRequest<Data> = bincode::deserialize(&data)?;

        self.addr.do_send(payload);

        Ok(())
    }

    fn handle_vote_request(&self, data: Vec<u8>) -> Result<(), WebServerError> {
        let payload: messages::VoteRequest = bincode::deserialize(&data)?;

        self.addr.do_send(payload);

        Ok(())
    }

    fn handle_install_snapshot_request(&self, data: Vec<u8>) -> Result<(), WebServerError> {
        let payload: messages::InstallSnapshotRequest = bincode::deserialize(&data)?;

        self.addr.do_send(payload);

        Ok(())
    }
}

use rocket;

impl WebServer {
    fn new(addr: Addr<AppRaft>) -> Self {
        let post_handler = PostHandler {
            addr: addr.clone()
        };

        let post_client_payload_route = Route::new(Method::Post, "/client/clientPayload", post_handler.clone());
        let append_entries_route = Route::new(Method::Post, "rpc/appendEntriesRequest", post_handler.clone());
        let vote_request_route = Route::new(Method::Post, "/rpc/voteRequest", post_handler.clone());
        let install_snapshot_route = Route::new(Method::Post, "/rpc/installSnapshotRequest", post_handler);

        let rocket = rocket::ignite().mount("/", vec![post_client_payload_route]);

        Self {
            addr,
            shared_network: SharedNetworkState::new(),
            rocket
        }
    }
}
