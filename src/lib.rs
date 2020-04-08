//#![deny(warnings)]
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
use std::sync::mpsc::channel;
use actix::fut::result;
use crate::futures::Future;

mod error;
mod config;
mod discovery;
mod node;
mod raft;
mod rpc;
mod startup;
mod http_helper;

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

    let node_id = rand::random();

    info!("Node config: {:?}", node_config);

    let shared_network_state = node::SharedNetworkState::new();

    shared_network_state.register_node(
        node_id,
        node_config.name.clone(),
        node_config.webserver.host.clone(),
        node_config.webserver.port
    );

    let admin_api = rpc::AdminNetwork::new();

    // Build the actix system.
    let mut sys = actix::System::new("my-awesome-app");

    // Build the needed runtime config for Raft specifying where
    // snapshots will be stored. See the storage chapter for more details.
    let config = RaftConfig::build(String::from("./snapshots")).validate().unwrap();

    /* match (opts.node_id, node_config.node_id) {
        (Some(n), _) => n,
        (_, Some(n)) => n,
        _ => 
    };*/

    info!("This node's ID is {}", node_id);

    let bootstrap_hosts = node_config.bootstrap_hosts.clone();

    let needs_to_join = !node_config.new_cluster;

    let known_nodes = if needs_to_join {
        if bootstrap_hosts.len() == 0 {
            error!("Need at least one bootstrap host");

            std::process::exit(1);
        } else {
            info!("Bootstrap hosts: {:?}", bootstrap_hosts);
        }

        let mut nodes = vec![];
        
        for host in bootstrap_hosts {
            info!("Getting members from {}", host);

            let new_nodes = admin_api.get_nodes(format!("http://{}/client/nodes", host).parse().unwrap()).unwrap();

            for n in new_nodes.iter() {
                shared_network_state.register_node(n.id, n.name.clone(), n.host.clone(), n.port);
            }

            nodes.append(&mut new_nodes.into_iter().map(|n| n.id).collect());
        }

        nodes
    } else {
        info!("Going to start a new cluster");
        vec![]
    };

    // Start off with just a single node in the cluster. Applications
    // should implement their own discovery system. See the cluster
    // formation chapter for more details.
    let mut members = if needs_to_join { 
        vec![node_id] //bootstrap_nodes[0].id, 
    } else {
        vec![node_id] 
    }; 

    for n in known_nodes {
        members.push(n);
    }

    /*for node in node_config.bootstrap_nodes.iter() {
        members.push(node.id);
    }*/

    let non_voters = vec![];

    let membership: messages::MembershipConfig = messages::MembershipConfig {
        is_in_joint_consensus: false,
        members: members.clone(),
        non_voters: non_voters,
        removing: vec![],
    };

    info!("Existing members: {:?}", members);

    let init_with_config = admin::InitWithConfig {
        members: vec![node_id]
    };

    let (sndr, recv) = channel();

    // Start the various actor types and hold on to their addrs.
    let network = raft::AppNetwork::new(shared_network_state.clone(), node_id, &node_config.webserver, sndr);
    let storage = raft::AppStorage::new(shared_network_state.clone(), membership);
    let metrics = raft::AppMetrics {};
    let network_addr = network.start();

    let app_raft = AppRaft::new(node_id, config, network_addr.clone(), storage.start(), metrics.start().recipient());

    let port = node_config.webserver.port;

    let app_raft_address = app_raft.start();

    let raft_addr = app_raft_address.clone();

    let startup = startup::StartupActor { node_id, raft_addr: raft_addr.clone(), admin_api };

    let startup_addr = startup.start();

    let mut webserver = rpc::WebServer::new(port, app_raft_address.clone(), shared_network_state.clone(), node_id);

    let mut admin_network = rpc::AdminNetwork::new();

    std::thread::spawn(move || webserver.start(node_id));

    std::thread::spawn(move || {
        use std::collections::BTreeMap;
        use crate::node::{NodeStateMachine};
        use std::time::Duration;

        let mut nodes: BTreeMap<u64, _>  = BTreeMap::new();

        for node_event in recv.iter() {
            let node_id = node_event.node.id;
            let is_err = {
                let entry = nodes.entry(node_event.node.id);
                let state_machine = entry.or_insert(NodeStateMachine::new(Duration::new(5, 0)));
                state_machine.transition(node_event);

                info!("Node state for {}: {:?}", node_id, state_machine);

                state_machine.is_err()

            };

            if is_err {
                use crate::futures::Future;

                error!("Node {} has FAILED!", node_id);
                match raft_addr.send(admin::ProposeConfigChange::new(vec![], vec![node_id])).wait() {
                    Ok(Ok(info)) => { 
                        nodes.remove(&node_id);
                        info!("Successfully removed node {} from the cluster: {:?}", node_id, info);
                    },
                    Ok(Err(admin::ProposeConfigChangeError::InoperableConfig)) => {
                        error!("Removing node would leave cluster inoperable");
                    },
                    Ok(Err(admin::ProposeConfigChangeError::Noop)) => {
                        nodes.remove(&node_id);
                        info!("Node already removed");
                    },
                    Ok(Err(admin::ProposeConfigChangeError::NodeNotLeader(_))) => {
                        nodes.remove(&node_id);
                        error!("This node is no longer leader")
                    },
                    Ok(Err(err)) => error!("Error removing node: {:?}", err),
                    Err(err) => error!("Utter failure removing node: {:?}", err),
                };
            }
        }
    });

    info!("Starting up...");

    let startup_fut = {
        let msg = if node_config.new_cluster {
            startup::StartupRequest::NewCluster { config: node_config, cluster_config: startup::ClusterConfig { } }
        } else {
            startup::StartupRequest::ExistingCluster { config: node_config }
        };

        startup_addr.send(msg)/* {
            Ok(res) => info!("Successfully started up: {:?}", res),
            Err(err) => {
                error!("Failed to start up: {:?}", err);

                std::process::exit(1)
            }
        }*/
    };

    //use crate::futures::Future;

    /*let res = startup_addr.send(msg);*/

    match sys.block_on(startup_fut) {
        Ok(res) => info!("Successfully started up: {:?}", res),
        Err(err) => { 
            error!("Failed to start up: {:?}", err);

            std::process::exit(1);
        }
    }

    match sys.run() {
        Err(err) => error!("Error in runtime: {:?}", err),
        Ok(_) => {}

    };
}
