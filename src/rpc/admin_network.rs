use log::{debug, error};
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use reqwest::blocking::Body;
use std::io::Cursor;

pub struct AdminNetwork;

impl AdminNetwork {
    pub fn new() -> Self {
        Self
    }

    pub fn session_request(&mut self, msg: CreateSessionRequest) -> Result<CreateSessionResponse, ()> {
        debug!("Handling CreateSessionRequest: {:?}", msg);

        match self.post_to_uri(&format!("http://{}:{}/client/createSessionRequest", msg.dest_node.address, msg.dest_node.port), msg) {
            Ok(resp) => Ok(resp),
            Err(err) => { 
                error!("Error making session request: {:?}", err);
                Err(())
            }
        }
    }

    fn post_to_uri<'r, S: Serialize + Debug, D: DeserializeOwned + Debug>(&self, uri: &str, msg: S) -> Result<D, reqwest::Error> {
        //let uri = format!("http://{}:{}{}", node.address, node.port, path);
        debug!("POST to {}", uri); 
        debug!("Serializing: {:?}", msg);

        let body_bytes: Vec<u8>  = bincode::serialize(&msg).unwrap();
        let body: Body = body_bytes.into();

        let response_result = reqwest::blocking::Client::new().post(uri)
            .header("User-Agent", "Membership-Raft")
            //.header("X-Node-Id", self.node_id)
            //.header("X-Node-Port", self.webserver.port)
            .body(body)
            .send();

        match response_result {
            Ok(resp) => {
                let bytes = resp.bytes().unwrap().into_iter().collect::<Vec<u8>>();
                debug!("Deserializing {} bytes from {}", bytes.len(), uri);

                let response: D = bincode::deserialize_from(Cursor::new(bytes)).unwrap();
                debug!("Deserialized: {:?}", response);

                Ok(response)
            },
            Err(err) => {
                error!("Error in response: {:?}", err);

                Err(err)
            }
        }
    }
}
