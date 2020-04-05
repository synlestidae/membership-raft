use crate::node::AppNode;
use std::time::Instant;
use std::error::Error;

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
            event_type: NodeEventType::Ok
        }
    }

    pub fn err<E: Error + 'static>(node: &AppNode, _err: E) -> Self {
        Self {
            timestamp: Instant::now(),
            node: node.clone(),
            event_type: NodeEventType::Err
        }
    }
}

#[derive(Clone, Debug)]
pub enum NodeEventType {
    Ok,
    Err//(Box<dyn Error + Clone + std::fmt::Debug>)
}
