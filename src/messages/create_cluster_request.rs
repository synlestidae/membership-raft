use crate::node::AppNode;
use crate::raft::RaftFut;
use actix::Message;
use actix_raft::NodeId;

#[derive(Debug)]
pub struct CreateClusterRequest {
    pub this_node: AppNode,
}

impl Message for CreateClusterRequest {
    type Result = Result<(), ()>;
}
