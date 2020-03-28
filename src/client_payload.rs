use crate::Data;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientPayload {
    pub data: Data,
}
