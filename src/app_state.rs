use serde::{Deserialize, Serialize};
use crate::app_node::AppNode;
use crate::Data;

#[derive(Deserialize, Serialize)]
pub struct AppState {
    nodes: Vec<AppNode>
}

impl AppState {
    pub fn apply(&mut self, transition: Data) {
        match transition {
            Data::AddNode { id, name, address, port } => self.nodes.push(AppNode {
                id,
                name, 
                address,
                port
            })
        }

        unimplemented!()
    }
}
