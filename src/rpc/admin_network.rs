use log::{debug, error};
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use reqwest::blocking::Body;
use std::io::Cursor;
use std::convert::TryInto;
use reqwest::Url;

pub struct AdminNetwork;

impl AdminNetwork {
    pub fn new() -> Self {
        Self
    }

    pub fn session_request(&mut self, url: Url, msg: CreateSessionRequest) -> Result<CreateSessionResponse, ()> {
        debug!("Handling CreateSessionRequest: {:?}", msg);

        match self.post_to_uri(url, msg) {
            Ok(resp) => Ok(resp),
            Err(err) => { 
                error!("Error making session request: {:?}", err);
                Err(())
            }
        }
    }

    fn post_to_uri<'r, S: Serialize + Debug, D: DeserializeOwned + Debug>(&self, uri: Url, msg: S) -> Result<D, reqwest::Error> {
        debug!("POST to {}", uri); 
        debug!("Serializing: {:?}", msg);

        let body_bytes: Vec<u8>  = bincode::serialize(&msg).unwrap();
        let body: Body = body_bytes.into();

        let response_result = reqwest::blocking::Client::new().post(uri.clone())
            .header("User-Agent", "Membership-Raft")
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
