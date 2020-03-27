use actix_raft::storage::GetInitialState;
use actix::{Actor, Context, ResponseActFuture, Handler};
use actix_raft::storage;
use actix_raft::messages;
use crate::error::Error;

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
