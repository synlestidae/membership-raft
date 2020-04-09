use crate::node::AppNode;
use actix_raft::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub new_node: AppNode,
}

impl actix::Message for CreateSessionRequest {
    type Result = Result<CreateSessionResponse, ()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreateSessionResponse {
    Error,
    RedirectToLeader { leader_node: AppNode },
    Success { leader_node_id: NodeId },
}
