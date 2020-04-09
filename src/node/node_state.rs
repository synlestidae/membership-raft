use crate::node::{NodeEvent, NodeEventType};
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum NodeState {
    Ok,
    TentativeErr { timestamp: Instant },
    TentativeOk { timestamp: Instant },
    Err,
}

#[derive(Clone, Debug)]
pub struct NodeStateMachine {
    pub node_state: NodeState,
    pub timeout: Duration,
}

impl NodeStateMachine {
    pub fn new(timeout: Duration) -> Self {
        Self {
            node_state: NodeState::Ok,
            timeout,
        }
    }

    pub fn transition(&mut self, event: NodeEvent) {
        let new_state = match (event.event_type, self.node_state.clone()) {
            (NodeEventType::Ok, NodeState::Ok) => NodeState::Ok,
            (NodeEventType::Err, NodeState::Ok) => NodeState::TentativeErr {
                timestamp: Instant::now(),
            },
            (NodeEventType::Err, NodeState::TentativeOk { timestamp }) => {
                NodeState::TentativeErr { timestamp }
            }
            (NodeEventType::Err, NodeState::TentativeErr { timestamp }) => {
                if self.timeout(timestamp) {
                    NodeState::Err
                } else {
                    NodeState::TentativeErr { timestamp }
                }
            }
            (NodeEventType::Ok, NodeState::TentativeErr { timestamp }) => {
                NodeState::TentativeOk { timestamp }
            }
            (NodeEventType::Ok, NodeState::TentativeOk { timestamp }) => {
                if self.timeout(timestamp) {
                    NodeState::Ok
                } else {
                    NodeState::TentativeOk { timestamp }
                }
            }
            (_, NodeState::Err) => NodeState::Err,
        };

        self.node_state = new_state;
    }

    fn timeout(&self, timestamp: Instant) -> bool {
        Instant::now().duration_since(timestamp) > self.timeout
    }

    pub fn is_err(&self) -> bool {
        match self.node_state {
            NodeState::Err => true,
            _ => false,
        }
    }
}
