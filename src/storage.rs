use crate::error::Error;
use crate::tokio::io::AsyncReadExt;
use crate::Data;
use crate::DataResponse;
use actix::fut::result;
use actix::fut::IntoActorFuture;
use actix::fut::WrapFuture;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages;
use actix_raft::storage;
use actix_raft::storage::GetInitialState;
use tokio::fs;

pub struct AppStorage {
    logs: Vec<messages::Entry<Data>>,
}

impl Actor for AppStorage {
    type Context = Context<Self>;
}

impl storage::RaftStorage<Data, DataResponse, Error> for AppStorage {
    type Actor = Self;

    type Context = Context<Self>;
}

impl Handler<storage::GetInitialState<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, storage::InitialState, Error>;

    fn handle(
        &mut self,
        _msg: storage::GetInitialState<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::new(result(Ok(storage::InitialState {
            last_log_index: 0,
            last_log_term: 0,
            last_applied_log: 0,
            hard_state: storage::HardState {
                current_term: 0,
                voted_for: None,
                membership: messages::MembershipConfig {
                    is_in_joint_consensus: false,
                    members: vec![],
                    non_voters: vec![],
                    removing: vec![],
                },
            },
        })))
    }
}

impl Handler<storage::SaveHardState<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::SaveHardState<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::new(result(Ok(())))
    }
}

impl Handler<storage::GetLogEntries<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, Vec<messages::Entry<Data>>, Error>;

    fn handle(
        &mut self,
        msg: storage::GetLogEntries<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::new(result(Ok(self
            .logs
            .iter()
            .skip(msg.start as usize)
            .take(msg.stop as usize - msg.start as usize)
            .cloned()
            .collect())))
    }
}

impl Handler<storage::AppendEntryToLog<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::AppendEntryToLog<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::ReplicateToLog<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::ReplicateToLog<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::ApplyEntryToStateMachine<Data, DataResponse, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, DataResponse, Error>;

    fn handle(
        &mut self,
        _msg: storage::ApplyEntryToStateMachine<Data, DataResponse, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::ReplicateToStateMachine<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::ReplicateToStateMachine<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::CreateSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, storage::CurrentSnapshotData, Error>;

    fn handle(
        &mut self,
        _msg: storage::CreateSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::InstallSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::InstallSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::GetCurrentSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, Option<storage::CurrentSnapshotData>, Error>;

    fn handle(
        &mut self,
        _msg: storage::GetCurrentSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // ... snip ...
        unimplemented!()
    }
}
