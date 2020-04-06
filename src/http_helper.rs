use log::{debug, error};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use reqwest::r#async::{Client, Body};
use std::io::Cursor;
use reqwest::Url;
use crate::futures::Future;
use crate::futures::Stream;
use reqwest::r#async::Chunk;

pub type HttpFuture<E, R> = Box<dyn Future<Item=E, Error=R>>;

pub struct HttpHelper;

impl HttpHelper {
    pub fn new() -> Self {
        Self
    }

    pub fn post_to_uri<S: Serialize + Debug, D: 'static + DeserializeOwned + Debug>(&self, uri: Url, msg: S) -> HttpFuture<D, ()> {
        debug!("POST to {}", uri); 
        debug!("Serializing: {:?}", msg);

        let body_bytes: Vec<u8>  = bincode::serialize(&msg).unwrap();
        let body: Body = body_bytes.into();

        let response_result = Client::new().post(uri.clone())
            .header("User-Agent", "Membership-Raft")
            .body(body)
            .send();

        Box::new(
            response_result.then(|result| {
                match result {
                    Ok(response) => {
                        Box::new(
                            response.into_body()
                                .collect()
                                .map(|chunks : Vec<Chunk>| {
                                    let bytes: Vec<u8> = chunks.into_iter().map(|c| c.as_ref().to_vec()).flatten().collect();

                                    debug!("Deserializing {} bytes", bytes.len());

                                    let obj: D = bincode::deserialize_from(Cursor::new(bytes)).unwrap();

                                    obj
                                })
                                .map_err(|_| ())
                        ) as HttpFuture<D, ()>
                    },
                    Err(err) => {
                        error!("Error in HTTP request: {:?}", err);
                        Box::new(futures::future::err(())) as HttpFuture<D, ()>
                    }
                }
            }).map_err(|err| {
                error!("Error in response: {:?}", err);

                err
            })
        )
    }
}
