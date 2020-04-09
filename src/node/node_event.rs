use crate::node::AppNode;
use std::error::Error;
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

#[derive(Clone, Debug)]
pub enum NodeEventType {
    Ok,
    Err, //(Box<dyn Error + Clone + std::fmt::Debug>)
}
