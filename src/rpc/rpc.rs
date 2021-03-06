use reqwest::Url;
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use futures::future::Future;
use crate::node::AppNode;
use actix_raft::messages;
use actix_raft;
use actix;

#[derive(Debug)]
pub enum RpcError {
    Request,
    Actix(actix::Error),
    StatusCode,
}

pub trait RpcClient {
    type JoinClusterFut: Future<Item=CreateSessionResponse, Error=RpcError>;
    type GetNodesFut: Future<Item=Vec<AppNode>, Error=RpcError>;
    type AppendEntriesFut: Future<Item=messages::AppendEntriesResponse, Error=RpcError>;
    type VoteFut: Future<Item=messages::VoteResponse, Error=RpcError>;
    type InstallSnapshotFut: Future<Item=messages::InstallSnapshotResponse, Error=RpcError>;

    fn join_cluster(&self, url: &Url, msg: CreateSessionRequest) -> Self::JoinClusterFut;

    fn get_nodes(&self, url: &Url) -> Self::GetNodesFut;

    fn append_entries(&self, url: &Url, msg: messages::AppendEntriesRequest<crate::raft::Transition>) -> Self::AppendEntriesFut;

    fn vote(&self, url: &Url, msg: messages::VoteRequest) -> Self::VoteFut;

    fn install_snapshot(&self, url: &Url, msg: messages::InstallSnapshotRequest) -> Self::InstallSnapshotFut;
}
