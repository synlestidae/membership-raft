use crate::node::AppNode;
use bincode;
use serde;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize, Serializer};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct SharedNetworkState {
    #[serde(
        serialize_with = "nodes_serialize",
        deserialize_with = "nodes_deserialize"
    )]
    nodes: Arc<Mutex<Vec<AppNode>>>,
}

fn nodes_deserialize<'r, D: Deserializer<'r>>(
    deserializer: D,
) -> Result<Arc<Mutex<Vec<AppNode>>>, D::Error> {
    //<D: Deserializer>(derializer: D) -> Result<D::Ok, D::Error> {
    let bytes: Vec<u8> = serde::de::Deserialize::deserialize(deserializer)?;
    let array: Vec<AppNode> = bincode::deserialize(&bytes).map_err(serde::de::Error::custom)?;

    Ok(Arc::new(Mutex::new(array)))
}

fn nodes_serialize<S: Serializer>(
    nodes: &Arc<Mutex<Vec<AppNode>>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(&bincode::serialize(&*nodes.lock().unwrap()).unwrap())
}

impl SharedNetworkState {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn register_node<S: ToString, T: ToString>(&self, id: u64, name: S, host: T, port: u16) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.retain(|n| n.id != id);

        nodes.push(AppNode {
            id,
            name: name.to_string(),
            host: host.to_string(),
            port,
        });
    }

    pub fn get_node(&self, id: u64) -> Option<AppNode> {
        self.nodes
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .find(|n| n.id == id)
    }

    pub fn get_nodes(&self) -> Vec<AppNode> {
        self.nodes.lock().unwrap().iter().cloned().collect()
    }
}
