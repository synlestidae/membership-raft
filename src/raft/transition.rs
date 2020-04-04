use serde::{Deserialize, Serialize};
use actix_raft;

/// The application's data type.
///
/// Enum types are recommended as typically there will be different types of data mutating
/// requests which will be submitted by application clients.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Transition {
    // Your data variants go here.
    AddNode { id: u64, name: String, address: std::net::IpAddr, port: u16 },
}

impl actix_raft::AppData for Transition {
}