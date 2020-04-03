use crate::raft::Transition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientPayload {
    pub data: Transition
}
