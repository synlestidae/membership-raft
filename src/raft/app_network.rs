use crate::node::NodeTracker;
use crate::node::SharedNetworkState;
use crate::raft::Transition;
use crate::rpc::HttpRpcClient;
use crate::rpc::RpcClient;
use actix;
use actix::{Actor, Context, Handler};
use actix_raft::messages::InstallSnapshotRequest;
use actix_raft::messages::InstallSnapshotResponse;
use actix_raft::messages::VoteRequest;
use actix_raft::messages::VoteResponse;
use actix_raft::NodeId;
use actix_raft::{messages, RaftNetwork};
use futures::Future;
use log::debug;

/// Your application's network interface actor.
pub struct AppNetwork {
    shared_network_state: SharedNetworkState,
    node_id: NodeId,
    node_tracker: actix::Addr<crate::NodeTracker>,
    rpc_client: HttpRpcClient,
}

impl AppNetwork {
    pub fn new(
        shared_network_state: SharedNetworkState,
        node_id: NodeId,
        node_tracker: actix::Addr<NodeTracker>,
    ) -> Self {
        Self {
            shared_network_state,
            node_id,
            node_tracker,
            rpc_client: HttpRpcClient::new(),
        }
    }
}

impl Actor for AppNetwork {
    type Context = Context<Self>;

    // ... snip ... other actix methods can be implemented here as needed.
}

// Ensure you impl this over your application's data type. Here, it is `Data`.
impl RaftNetwork<Transition> for AppNetwork {}

type AppNetworkFut<E, R> = Box<dyn Future<Item = E, Error = R>>;

// Then you just implement the various message handlers.
// See the network chapter for details.
impl Handler<messages::AppendEntriesRequest<Transition>> for AppNetwork {
    type Result = AppNetworkFut<messages::AppendEntriesResponse, ()>;

    fn handle(
        &mut self,
        msg: messages::AppendEntriesRequest<Transition>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let node_option = self.shared_network_state.get_node(msg.target);

        //let addr = self.

        if let Some(node) = node_option {
            let node_tracker_addr = self.node_tracker.clone();

            Box::new(
                self.rpc_client
                    .append_entries(&node.rpc_url(), msg)
                    .then(move |result| {
                        /*match result {
                            Ok(response) => node_tracker_addr.send(NodeEvent::ok(&node)),
                            Err(err) => node_tracker_addr.send(NodeEvent::err(&node))
                        };*/

                        //Box::new(fut.then(|_| result))

                        result
                    })
                    .map_err(|_| ()),
            )
        } else {
            panic!("Unsure where to send this")
        }
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = AppNetworkFut<InstallSnapshotResponse, ()>;

    fn handle(&mut self, msg: InstallSnapshotRequest, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling InstallSnapshotRequest: {:?}", msg);

        let node = self
            .shared_network_state
            .get_node(msg.target)
            .expect("Unable to find node");

        Box::new(
            self.rpc_client
                .install_snapshot(&node.rpc_url(), msg)
                .map_err(|_| ()),
        )
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<VoteRequest> for AppNetwork {
    type Result = AppNetworkFut<VoteResponse, ()>;

    fn handle(&mut self, msg: VoteRequest, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling VoteRequest: {:?}", msg);

        let node = self
            .shared_network_state
            .get_node(msg.target)
            .expect("Unable to find node");

        Box::new(self.rpc_client.vote(&node.rpc_url(), msg).map_err(|_| ()))
    }
}
