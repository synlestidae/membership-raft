use crate::node::AppNode;
use crate::raft::RaftFut;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct NodeEvent {
    pub timestamp: Instant,
    pub node: AppNode,
    pub event_type: NodeEventType,
}

impl NodeEvent {
    pub fn ok(node: &AppNode) -> Self {
        Self {
            timestamp: Instant::now(),
            node: node.clone(),
            event_type: NodeEventType::Ok,
        }
    }

    pub fn err(node: &AppNode) -> Self {
        Self {
            timestamp: Instant::now(),
            node: node.clone(),
            event_type: NodeEventType::Err,
        }
    }
}

impl actix::Message for NodeEvent {
    type Result = RaftFut<(), ()>;
}

#[derive(Clone, Debug)]
pub enum NodeEventType {
    Ok,
    Err,
}
