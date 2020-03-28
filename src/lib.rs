extern crate actix;
extern crate actix_raft;
extern crate bincode;
extern crate serde;
extern crate tokio;

use actix_raft::{AppData, AppDataResponse};
use serde::{Deserialize, Serialize};

mod error;
mod network;
mod storage;

/// The application's data type.
///
/// Enum types are recommended as typically there will be different types of data mutating
/// requests which will be submitted by application clients.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Data {
    // Your data variants go here.
}

/// The application's data response types.
///
/// Enum types are recommended as typically there will be multiple response types which can be
/// returned from the storage layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum DataResponse {
    // Your response variants go here.
    Success { msg: String }
}

impl DataResponse {
    pub fn success<S: ToString>(s: S) -> Self {
        DataResponse::Success { msg: s.to_string() }
    }
}

/// This also has a `'static` lifetime constraint, so no `&` references at this time.
/// The new futures & async/await should help out with this quite a lot, so
/// hopefully this constraint will be removed in actix as well.
impl AppData for Data {}

/// This also has a `'static` lifetime constraint, so no `&` references at this time.
impl AppDataResponse for DataResponse {}
