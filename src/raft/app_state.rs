use crate::node::AppNode;
use serde::{Deserialize, Serialize};
//use crate::raft::Transition;

#[derive(Deserialize, Serialize)]
pub struct AppState {
    nodes: Vec<AppNode>,
}

impl AppState {
    /*pub fn apply(&mut self, transition: Transition) {
        match transition {
            Transition::AddNode { id, name, address, port } => self.nodes.push(AppNode {
                id,
                name,
                address,
                port
            })
        }
    }*/
}
