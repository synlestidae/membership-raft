extern crate actix;
extern crate actix_raft;
extern crate bincode;
extern crate serde;
extern crate tokio;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate toml;
extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate rand;
extern crate futures_util;
extern crate reqwest;
extern crate futures;
extern crate tarpc;

use actix_raft::Raft;

use actix_raft::{AppData, AppDataResponse};
use actix_raft::messages;
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
mod config;
mod rpc;
mod server;
mod create_session_request;

/// The application's data type.
///
/// Enum types are recommended as typically there will be different types of data mutating
/// requests which will be submitted by application clients.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Data {
    // Your data variants go here.
    AddNode { id: u64, name: String, address: std::net::IpAddr, port: u16 },
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

use actix_raft::{Config as RaftConfig, ConfigBuilder, SnapshotPolicy};

use crate::actix::Actor;

use actix_raft::admin;

use clap::Clap;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Mate Antunovic")]
struct Opts {
    #[clap(short = "c", long = "config")]
    pub config: Option<String>,

    #[clap(short = "n", long = "nodeid")]
    pub node_id: Option<u64>,
}

use log::{info, error, debug};

fn main() {
    simple_logger::init().unwrap();
    let opts: Opts = Opts::parse();

    info!("Opts: {:?}", opts);

    let node_config: config::Config = match opts.config { 
        Some(config_path) => {
            info!("Loading config from {}", config_path);
            let config_string = std::fs::read_to_string(config_path).unwrap();

            toml::from_str(&config_string).unwrap()
        },
        None => {
            info!("Not using config");
            Default::default()
        }
    };

    info!("Node config: {:?}", node_config);

    let mut shared_network_state = shared_network_state::SharedNetworkState::new();

    for n in node_config.bootstrap_nodes.iter() {
        info!("Bootstrap node: {:?}", n);
        shared_network_state.register_node(n.id, &n.name, n.address, n.port);
    }

    // Build the actix system.
    let sys = actix::System::new("my-awesome-app");

    // Build the needed runtime config for Raft specifying where
    // snapshots will be stored. See the storage chapter for more details.
    let config = RaftConfig::build(String::from("./snapshots")).validate().unwrap();

    let node_id = match (opts.node_id, node_config.node_id) {
        (Some(n), _) => n,
        (_, Some(n)) => n,
        _ => rand::random()
    };

    info!("This node's ID is {}", node_id);

    // Start off with just a single node in the cluster. Applications
    // should implement their own discovery system. See the cluster
    // formation chapter for more details.
    let mut members = vec![node_id];

    for node in node_config.bootstrap_nodes.iter() {
        members.push(node.id);
    }

    let membership: messages::MembershipConfig = messages::MembershipConfig {
        is_in_joint_consensus: false,
        members: members.clone(),
        non_voters: vec![],
        removing: vec![],
    };

    let init_with_config = admin::InitWithConfig {
        members: node_config.bootstrap_nodes.iter().map(|n| n.id).collect()
    };

    // Start the various actor types and hold on to their addrs.
    let network = network::AppNetwork::new(shared_network_state.clone(), node_id, &node_config.webserver);
    let storage = storage::AppStorage::new(shared_network_state, membership);
    let metrics = metrics::AppMetrics::new();

    let network_addr = network.start();

    let app_raft = AppRaft::new(node_id, config, network_addr.clone(), storage.start(), metrics.start().recipient());

    let app_raft_address = app_raft.start();

    let app_raft_address2 = app_raft_address.clone();
    let port = node_config.webserver.port;

    let mut webserver = webserver::WebServer::new(port, app_raft_address.clone());

    std::thread::spawn(move || {
        const SECONDS_DELAY: u64 = 5;

        info!("Waiting for {} seconds before adding config", SECONDS_DELAY);

        std::thread::sleep(std::time::Duration::new(SECONDS_DELAY, 0));

        info!("Adding config");

        match app_raft_address.try_send(init_with_config) {
            Ok(()) => info!("Successfully added config"),
            Err(err) => error!("Error adding config: {:?}", err)
        };

        webserver.start();
    });

    let config_name = node_config.name;

    info!("Server will run on port {}", node_config.webserver.port);

    let needs_to_join = !node_config.node_id.is_some();

    let bootstrap_nodes = node_config.bootstrap_nodes.clone(); // [0].clone();

    std::thread::spawn(move || {
        const SECONDS_DELAY: u64 = 4;

        info!("Waiting for {} seconds before registering node message", SECONDS_DELAY);

        std::thread::sleep(std::time::Duration::new(SECONDS_DELAY, 0));

        info!("Registering this node: {}", needs_to_join);

        if needs_to_join {
            debug!("Attempting to join cluster");
            use crate::futures::Future;

            let response_result = network_addr.send(crate::create_session_request::CreateSessionRequest { 
                new_node: crate::app_node::AppNode {
                    id: node_id,
                    address: "127.0.0.1".parse().unwrap(),
                    name: String::from("test-node"),
                    port
                },
                dest_node: bootstrap_nodes[0].clone(),
                //node_id: bootstrap_nodes[0].id 
            }).wait();

            info!("Response result: {:?}", response_result);

        //info!("Info: {:?}", response);
        }

        match app_raft_address2.try_send(admin::ProposeConfigChange::new(vec![node_id], vec![])) {
            Ok(()) => info!("Successfully sent message to register Node"),
            Err(err) => error!("Error sending register Node message {:?}", err)
        };
    });

    // Run the actix system. Unix signals for termination &
    // graceful shutdown are automatically handled.
    let _ = sys.run();
}

