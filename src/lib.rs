//#![deny(warnings)]
extern crate actix;
extern crate actix_raft;
extern crate bincode;
extern crate clap;
extern crate futures;
extern crate futures_util;
extern crate log;
extern crate rand;
extern crate reqwest;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;
extern crate tarpc;
extern crate tokio;
extern crate toml;

use crate::actix::Actor;
use crate::clap::Clap;
use crate::futures::Future;
use crate::node::NodeTracker;
use actix_raft::Raft;
use log::{error, info};

mod config;
mod discovery;
mod error;
mod messages;
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
type AppRaft =
    Raft<raft::Transition, raft::DataResponse, error::Error, raft::AppNetwork, raft::AppStorage>;

pub fn main() {
    simple_logger::init().unwrap();
    let opts: config::Opts = config::Opts::parse();

    info!("Opts: {:?}", opts);

    let config: config::Config = match opts.config {
        Some(config_path) => {
            info!("Loading config from {}", config_path);
            let config_string = std::fs::read_to_string(config_path).unwrap();

            toml::from_str(&config_string).unwrap()
        }
        None => {
            info!("Not using config");
            Default::default()
        }
    };

    let node_id = rand::random();

    info!("This node's ID is {}", node_id);
    info!("Node config: {:?}", config);

    let shared_network_state = node::SharedNetworkState::new();

    shared_network_state.register_node(
        node_id,
        config.name.clone(),
        config.rpc_host.clone(),
        config.rpc_port,
    );

    // Build the actix system.
    let mut sys = actix::System::new("my-awesome-app");

    let mut builder = crate::raft::RaftBuilder::new(node_id);

    builder
        .snapshot_dir("./snapshots/")
        .bootstrap_hosts(config.bootstrap_hosts)
        .rpc_host(&config.rpc_host)
        .rpc_port(config.rpc_port);

    let nodes = if !config.is_new_cluster {
        let discovery = match builder.discovery() {
            Ok(d) => d,
            Err(err) => {
                error!("Could not build node: {:?}", err);

                std::process::exit(1)
            }
        };

        match sys.block_on(discovery.nodes()) {
            Ok(nodes) => nodes,
            Err(err) => {
                error!("Could not discover existing nodes: {:?}", err);

                std::process::exit(1);
            }
        }
    } else {
        vec![]
    };

    let this_node: crate::node::AppNode = node::AppNode {
        id: node_id,
        name: config.name.to_string(),
        host: config.rpc_host.to_string(),
        port: config.rpc_port,
    };

    let mut raft = builder.build();
    let app_addr = raft.activate();

    let raft_addr = raft.start();
    let is_new_cluster = config.is_new_cluster;

    info!("Starting the node");

    //sys.block_on(tokio::time::delay_for(tokio::time::Duration::new(2, 0)));

    if is_new_cluster {
        info!("COnnected? {}", raft_addr.connected());
        sys.block_on(
            raft_addr
                .send(messages::CreateClusterRequest { this_node })
                .map(|result| info!("Result from creating cluster: {:?}", result))
                .map_err(|err| {
                    error!("Error while creating cluster: {:?}", err);

                    std::thread::sleep(std::time::Duration::new(1, 0));

                    //std::process::exit(1);
                }),
        );
    } else {
        sys.block_on(
            raft_addr
                .send(messages::JoinClusterRequest { this_node, nodes })
                .map(|result| info!("Result from joining cluster: {:?}", result))
                .map_err(|err| {
                    error!("Error while joining existing cluster: {:?}", err);

                    std::thread::sleep(std::time::Duration::new(1, 0));

                    //std::process::exit(1);
                })
        );
    }

    match sys.block_on(app_addr.clone().send(actix_raft::admin::InitWithConfig::new(vec![node_id]))) {
        Ok(res) => info!("GOOD? {:?}", res),
        Err(err) => error!("BAD: {:?}", err)
    };


    info!("Starting runtime");

    match sys.run() {
        Err(err) => error!("Error in runtime: {:?}", err),
        Ok(r) => info!("Shutting down: {:?}", r),
    };

    drop(raft_addr);
}
