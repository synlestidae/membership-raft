use crate::app_node::AppNode;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct SharedNetworkState {
    nodes: Arc<Mutex<Vec<AppNode>>>
}

impl SharedNetworkState {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(vec![]))
        }
    }

    pub fn register_node(&mut self, id: u64, name: String, address: std::net::IpAddr, port: u16) {
        self.nodes.lock().unwrap().push(AppNode {
            id,
            name,
            address,
            port
        });
    }

    pub fn get_node(&mut self, id: u64) -> Option<AppNode> {
        self.nodes.lock().unwrap().iter().cloned().find(|n| n.id == id)
    }
}

