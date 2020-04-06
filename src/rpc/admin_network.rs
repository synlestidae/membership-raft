use log::{debug, error};
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use reqwest::r#async::{Client, Body};
use std::io::Cursor;
use std::convert::TryInto;
use reqwest::Url;
use crate::futures::Future;
use crate::futures::Stream;
use reqwest::r#async::Chunk;

pub struct AdminNetwork { 
    http_helper: crate::http_helper::HttpHelper
}

type AdminNetworkFut<E, R> = Box<dyn Future<Item=E, Error=R>>;

impl AdminNetwork {
    pub fn new() -> Self {
        Self {
            http_helper: crate::http_helper::HttpHelper::new()
        }
    }

    pub fn session_request(&mut self, url: Url, msg: CreateSessionRequest) -> AdminNetworkFut<CreateSessionResponse, ()> {
        debug!("Handling CreateSessionRequest: {:?}", msg);

        self.http_helper.post_to_uri::<CreateSessionRequest, CreateSessionResponse>(url, msg)
    }

    /*fn post_to_uri<S: Serialize + Debug, D: DeserializeOwned + Debug>(&self, uri: Url, msg: S) -> AdminNetworkFut<CreateSessionResponse, ()> {
        debug!("POST to {}", uri); 
        debug!("Serializing: {:?}", msg);

        let body_bytes: Vec<u8>  = bincode::serialize(&msg).unwrap();
        let body: Body = body_bytes.into();

        let response_result = Client::new().post(uri.clone())
            .header("User-Agent", "Membership-Raft")
            .body(body)
            .send();

        response_result.then(|result| {
            match result {
                Ok(response) => {
                    response.into_body()
                        .collect()
                        .map(|chunks : Vec<Chunk>| {
                            let bytes: Vec<u8> = chunks.into_iter().map(|c| c.as_ref().to_vec()).flatten().collect();

                            debug!("Deserializing {} bytes", bytes.len());

                            let obj: D = bincode::deserialize_from(Cursor::new(bytes)).unwrap();

                            obj
                        })
                },
                Err(_) => unimplemented!()
            }
        }).map_err(|err| {
            error!("Error in response: {:?}", err);

            err
        });

        unimplemented!()
    }*/
}
