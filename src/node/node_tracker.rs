use crate::node::NodeEvent;
use crate::node::NodeStateMachine;
use crate::raft::Raft;
use crate::raft::RaftFut;
use actix::Addr;
use actix_raft::NodeId;
use log::info;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub struct NodeTracker {
    nodes: BTreeMap<u64, NodeStateMachine>,
    raft: Option<Addr<Raft>>,
}

pub enum NodeAction {
    RemoveNode,
    KeepNode,
}

impl NodeTracker {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            raft: None as Option<Addr<Raft>>,
        }
    }

    pub fn raft(&mut self, raft: Addr<Raft>) {
        self.raft = Some(raft);
    }

    fn remove_node(nodes: Arc<Mutex<BTreeMap<u64, NodeStateMachine>>>, node_id: NodeId) {
        if let Ok(mut nodes) = nodes.lock() {
            nodes.remove(&node_id);
        }
    }

    pub fn event(&mut self, node_event: NodeEvent) -> NodeAction {
        let node_id = node_event.node.id;
        let entry = self.nodes.entry(node_event.node.id);
        let state_machine = entry.or_insert(NodeStateMachine::new(Duration::new(5, 0)));

        state_machine.transition(node_event);

        info!("Node state for {}: {:?}", node_id, state_machine);

        if state_machine.is_err() {
            NodeAction::RemoveNode
        } else {
            NodeAction::KeepNode
        }
    }
}

impl actix::Actor for NodeTracker {
    type Context = actix::Context<Self>;
}

impl actix::Handler<NodeEvent> for NodeTracker {
    type Result = <NodeEvent as actix::Message>::Result;

    fn handle(&mut self, _: NodeEvent, _: &mut Self::Context) -> RaftFut<(), ()> {
        todo!()
    }
}
