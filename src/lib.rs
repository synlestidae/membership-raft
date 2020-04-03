#![deny(warnings)]
extern crate actix;
extern crate actix_raft;
extern crate bincode;
extern crate serde;
extern crate tokio;
extern crate rocket;
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
use actix_raft::admin;
use actix_raft::messages;
use actix_raft::{Config as RaftConfig};
use crate::actix::Actor;
use crate::clap::Clap;
use log::{info, error};

mod error;
mod config;
mod discovery;
mod node;
mod raft;
mod rpc;

/// The application's data response types.
///
/// Enum types are recommended as typically there will be multiple response types which can be
/// returned from the storage layer.

/// This also has a `'static` lifetime constraint, so no `&` references at this time.
/// The new futures & async/await should help out with this quite a lot, so
/// hopefully this constraint will be removed in actix as well.


/// A type alias used to define an application's concrete Raft type.
type AppRaft = Raft<raft::Transition, raft::DataResponse, error::Error, raft::AppNetwork, raft::AppStorage>;

pub fn main() {
    simple_logger::init().unwrap();
    let opts: config::Opts = config::Opts::parse();

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

    let shared_network_state = node::SharedNetworkState::new();

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

    let bootstrap_nodes = node_config.bootstrap_nodes.clone();

    let needs_to_join = !node_config.node_id.is_some();

    // Start off with just a single node in the cluster. Applications
    // should implement their own discovery system. See the cluster
    // formation chapter for more details.
    let mut members = if needs_to_join { 
        vec![node_id] //bootstrap_nodes[0].id, 
    } else {
        vec![node_id] 
    }; 

    for node in node_config.bootstrap_nodes.iter() {
        members.push(node.id);
    }

    let non_voters = vec![];/*if needs_to_join {
        vec![node_id]
    } else {
        vec![]
    };*/

    let membership: messages::MembershipConfig = messages::MembershipConfig {
        is_in_joint_consensus: true,
        members: members.clone(),
        non_voters: non_voters,
        removing: vec![],
    };

    let init_with_config = admin::InitWithConfig {
        members: node_config.bootstrap_nodes.iter().map(|n| n.id).collect()
    };

    // Start the various actor types and hold on to their addrs.
    let network = raft::AppNetwork::new(shared_network_state.clone(), node_id, &node_config.webserver);
    let storage = raft::AppStorage::new(shared_network_state.clone(), membership);
    let metrics = raft::AppMetrics {};

    let network_addr = network.start();

    let app_raft = AppRaft::new(node_id, config, network_addr.clone(), storage.start(), metrics.start().recipient());

    let port = node_config.webserver.port;

    let app_raft_address = app_raft.start();

    let mut webserver = rpc::WebServer::new(port, app_raft_address.clone(), shared_network_state.clone());

    let mut admin_network = rpc::AdminNetwork::new();

    std::thread::spawn(move || webserver.start());

    std::thread::spawn(move || {
        if needs_to_join {
            std::thread::sleep(std::time::Duration::new(0, 1000));
            info!("Asking for a new session");

            let response_result = admin_network.session_request(rpc::CreateSessionRequest { 
                new_node: node::AppNode {
                    id: node_id,
                    address: "127.0.0.1".parse().unwrap(),
                    name: String::from("test-node"),
                    port
                },
                dest_node: bootstrap_nodes[0].clone(),
            }).unwrap();

            info!("Response result: {:?}", response_result);
        } else {
            std::thread::sleep(std::time::Duration::new(1, 0));

            info!("Starting cluster");

            match app_raft_address.send(init_with_config).wait() {
                Ok(r) => info!("Successfully added config: {:?}", r),
                Err(err) => error!("Error adding config: {:?}", err)
            };
        }
    });

    use crate::futures::Future;

    // Run the actix system. Unix signals for Committedtermination &
    // graceful shutdown are automatically handled.
    let _ = sys.run();
}
