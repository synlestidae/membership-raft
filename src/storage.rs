use actix_raft::storage::GetInitialState;
use actix::{Actor, Context, ResponseActFuture, Handler};
use actix_raft::storage;
use actix_raft::messages;
use crate::error::Error;
use crate::Data;
use crate::DataResponse;

pub struct AppStorage {
}

impl Actor for AppStorage {
    type Context = Context<Self>;
}

impl Handler<storage::GetInitialState<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, storage::InitialState, Error>;

    fn handle(&mut self, _msg: storage::GetInitialState<Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::SaveHardState<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(&mut self, _msg: storage::SaveHardState<Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::GetLogEntries<Data, Error>> for AppStorage {
    type Result =  ResponseActFuture<Self, Vec<messages::Entry<Data>>, Error>;

    fn handle(&mut self, _msg: storage::GetLogEntries<Data, Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::ReplicateToLog<Data, Error>> for AppStorage {
    type Result =  ResponseActFuture<Self, (), Error>;

    fn handle(&mut self, _msg: storage::ReplicateToLog<Data, Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::ApplyEntryToStateMachine<Data, DataResponse, Error>> for AppStorage {
    type Result =  ResponseActFuture<Self, DataResponse, Error>;

    fn handle(&mut self, _msg: storage::ApplyEntryToStateMachine<Data, DataResponse, Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::ReplicateToStateMachine<Data, Error>> for AppStorage {
    type Result =  ResponseActFuture<Self, (), Error>;

    fn handle(&mut self, _msg: storage::ReplicateToStateMachine<Data, Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::CreateSnapshot<Error>> for AppStorage {
    type Result =  ResponseActFuture<Self, storage::CurrentSnapshotData, Error>;

    fn handle(&mut self, _msg: storage::CreateSnapshot<Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::InstallSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(&mut self, _msg: storage::InstallSnapshot<Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::GetCurrentSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, Option<storage::CurrentSnapshotData>, Error>;

    fn handle(&mut self, _msg: storage::GetCurrentSnapshot<Error>, _ctx: &mut Self::Context) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}
