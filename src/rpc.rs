use tarpc;
use actix_raft::messages;
use crate::Data;

use std::net::ToSocketAddrs;

#[tarpc::service]
pub trait Rpc {
    async fn send_client_payload(msg: messages::AppendEntriesRequest<Data>) -> Result<messages::AppendEntriesResponse, ()>;
    //async fn send_install_snapshot(msg: messages::InstallSnapshotRequest) -> Result<messages::InstallSnapshotResponse, ()>;
    //async fn send_append_entries(msg: messages::AppendEntriesRequest<Data>) -> Result<messages::AppendEntriesResponse, ()>;
}
