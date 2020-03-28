extern crate actix;
extern crate actix_raft;
extern crate bincode;
extern crate serde;
extern crate tokio;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;

use actix_raft::Raft;

use actix_raft::{AppData, AppDataResponse};
use serde::{Deserialize, Serialize};

mod error;
mod network;
mod storage;
mod app_node;
mod shared_network_state;
mod webserver;
mod client_payload;
mod metrics;
mod app_state;

/// The application's data type.
///
/// Enum types are recommended as typically there will be different types of data mutating
/// requests which will be submitted by application clients.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Data {
    // Your data variants go here.
    AddNode { id: u64, name: String, address: std::net::IpAddr, port: u16 }
}

/// The application's data response types.
///
/// Enum types are recommended as typically there will be multiple response types which can be
/// returned from the storage layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum DataResponse {
    // Your response variants go here.
    Success { msg: String }
}

impl DataResponse {
    pub fn success<S: ToString>(s: S) -> Self {
        DataResponse::Success { msg: s.to_string() }
    }
}

/// This also has a `'static` lifetime constraint, so no `&` references at this time.
/// The new futures & async/await should help out with this quite a lot, so
/// hopefully this constraint will be removed in actix as well.
impl AppData for Data {
}

/// This also has a `'static` lifetime constraint, so no `&` references at this time.
impl AppDataResponse for DataResponse {}


/// A type alias used to define an application's concrete Raft type.
type AppRaft = Raft<Data, DataResponse, error::Error, network::AppNetwork, storage::AppStorage>;

use actix_raft::{Config, ConfigBuilder, SnapshotPolicy};

use crate::actix::Actor;

fn main() {
    // Build the actix system.
    let sys = actix::System::new("my-awesome-app");

    // Build the needed runtime config for Raft specifying where
    // snapshots will be stored. See the storage chapter for more details.
    let config = Config::build(String::from("/app/snapshots")).validate().unwrap();

    // Start off with just a single node in the cluster. Applications
    // should implement their own discovery system. See the cluster
    // formation chapter for more details.
    let members = vec![1];

    let shared_network_state = shared_network_state::SharedNetworkState::new();

    // Start the various actor types and hold on to their addrs.
    let network = network::AppNetwork::new(shared_network_state);
    let storage = storage::AppStorage::new();
    let metrics = metrics::AppMetrics::new();
    let app_raft = AppRaft::new(1, config, network.start(), storage.start(), metrics.start().recipient());

    // Run the actix system. Unix signals for termination &
    // graceful shutdown are automatically handled.
    let _ = sys.run();
}

