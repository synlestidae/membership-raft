use crate::Data;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages::InstallSnapshotRequest;
use actix_raft::messages::InstallSnapshotResponse;
use actix_raft::messages::VoteRequest;
use actix_raft::messages::VoteResponse;
use actix_raft::{messages, RaftNetwork};

/// Your application's network interface actor.
struct AppNetwork {/* ... snip ... */}

impl Actor for AppNetwork {
    type Context = Context<Self>;

    // ... snip ... other actix methods can be implemented here as needed.
}

// Ensure you impl this over your application's data type. Here, it is `Data`.
impl RaftNetwork<Data> for AppNetwork {}

/*impl Handler<AppendEntriesRequest<Data>> for AppNetwork {
    type Result = AppendEntriesRequest<Data>;

    fn handle(&mut self, msg: AppendEntriesRequest<Data>, ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}*/

/*impl Handler<VoteRequest> for AppNetwork {
    type Result = VoteRequest;

    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}*/

/*impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = InstallSnapshotRequest;

    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}*/

// Then you just implement the various message handlers.
// See the network chapter for details.
impl Handler<messages::AppendEntriesRequest<Data>> for AppNetwork {
    type Result = ResponseActFuture<Self, messages::AppendEntriesResponse, ()>;

    fn handle(
        &mut self,
        _msg: messages::AppendEntriesRequest<Data>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<InstallSnapshotRequest> for AppNetwork {
    type Result = ResponseActFuture<Self, InstallSnapshotResponse, ()>;

    fn handle(&mut self, _msg: InstallSnapshotRequest, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

// Impl handlers on `AppNetwork` for the other `actix_raft::messages` message types.
//
impl Handler<VoteRequest> for AppNetwork {
    type Result = ResponseActFuture<Self, VoteResponse, ()>;

    fn handle(&mut self, _msg: VoteRequest, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}
