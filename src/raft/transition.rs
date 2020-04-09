use actix_raft;
use serde::{Deserialize, Serialize};

/// The application's data type.
///
/// Enum types are recommended as typically there will be different types of data mutating
/// requests which will be submitted by application clients.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Transition {
    // Your data variants go here.
    AddNode {
        id: u64,
        name: String,
        host: String,
        port: u16,
    },
}

impl actix_raft::AppData for Transition {}
