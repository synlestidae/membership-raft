use crate::node::AppNode;
use crate::raft::RaftFut;
use actix::Message;

#[derive(Debug)]
pub struct JoinClusterRequest {
    pub this_node: AppNode,
    pub nodes: Vec<AppNode>,
}

impl Message for JoinClusterRequest {
    type Result = Result<(), ()>;
}
