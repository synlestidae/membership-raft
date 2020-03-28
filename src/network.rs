use crate::Data;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages::InstallSnapshotRequest;
use actix_raft::messages::InstallSnapshotResponse;
use actix_raft::messages::VoteRequest;
use actix_raft::messages::VoteResponse;
use actix_raft::{messages, RaftNetwork};
use std::net::IpAddr;
use crate::shared_network_state::SharedNetworkState;
use actix::fut::result;

/// Your application's network interface actor.
pub struct AppNetwork {
    shared_network_state: SharedNetworkState
}

impl AppNetwork {
    pub fn new(shared_network_state: SharedNetworkState) -> Self {
        Self {
            shared_network_state
        }
    }
}

impl Actor for AppNetwork {
    type Context = Context<Self>;

    // ... snip ... other actix methods can be implemented here as needed.
}

// Ensure you impl this over your application's data type. Here, it is `Data`.
impl RaftNetwork<Data> for AppNetwork {}

// Then you just implement the various message handlers.
// See the network chapter for details.
impl Handler<messages::AppendEntriesRequest<Data>> for AppNetwork {
    type Result = ResponseActFuture<Self, messages::AppendEntriesResponse, ()>;

    fn handle(
        &mut self,
        msg: messages::AppendEntriesRequest<Data>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let node_option = self.shared_network_state.get_node(msg.target);

        match node_option {
            Some(node) => {
            },
            None => return Box::new(result(Err(())))
        }
        // ... snip ...
        unimplemented!()
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = ResponseActFuture<Self, InstallSnapshotResponse, ()>;

    fn handle(&mut self, msg: InstallSnapshotRequest, _ctx: &mut Self::Context) -> Self::Result {
        let node_option = self.shared_network_state.get_node(msg.target);

        match node_option {
            Some(node) => {
            },
            None => return Box::new(result(Err(())))
        }
        // ... snip ...
        unimplemented!()
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<VoteRequest> for AppNetwork {
    type Result = ResponseActFuture<Self, VoteResponse, ()>;

    fn handle(&mut self, msg: VoteRequest, _ctx: &mut Self::Context) -> Self::Result {
        let node_option = self.shared_network_state.get_node(msg.target);

        match node_option {
            Some(node) => {
            },
            None => return Box::new(result(Err(())))
        }
        unimplemented!()
    }
}
