use crate::app_node::AppNode;
use actix_raft::NodeId;
use serde::{Deserialize, Serialize};
use actix_raft::admin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub new_node: AppNode,
    pub dest_node: AppNode,
    //pub node_id: NodeId
}

impl actix::Message for CreateSessionRequest {
    type Result = Result<CreateSessionResponse, ()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreateSessionResponse {
    Error,// { error: admin::ProposeConfigChangeError<crate::Data, crate::DataResponse, crate::error::Error> }, 
    RedirectToLeader { leader_node : AppNode },
    Success
}
