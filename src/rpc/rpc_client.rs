use crate::node::AppNode;
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use actix_raft;
use actix_raft::messages;
use futures::future::Future;
use log::error;
use reqwest::Url;

pub enum RpcError {
    Request,
    StatusCode,
}

impl From<reqwest::Error> for RpcError {
    fn from(err: reqwest::Error) -> Self {
        error!("RpcError: {:?}", err);

        RpcError::Request
    }
}

pub trait RpcClient {
    type JoinClusterFut: Future<Item = CreateSessionResponse, Error = RpcError>;
    type GetNodesFut: Future<Item = Vec<AppNode>, Error = RpcError>;
    type AppendEntriesFut: Future<Item = messages::AppendEntriesResponse, Error = RpcError>;
    type VoteFut: Future<Item = messages::VoteResponse, Error = RpcError>;
    type InstallSnapshotFut: Future<Item = messages::InstallSnapshotResponse, Error = RpcError>;

    fn join_cluster(&self, url: &Url, req: CreateSessionRequest) -> Self::JoinClusterFut;

    fn get_nodes(&self, url: &Url) -> Self::JoinClusterFut;

    fn append_entries(
        &self,
        url: &Url,
        msg: messages::AppendEntriesRequest<crate::raft::Transition>,
    ) -> Self::AppendEntriesFut;

    fn vote(&self, url: &Url, msg: messages::VoteRequest) -> Self::VoteFut;

    fn install_snapshot(
        &self,
        url: &Url,
        msg: messages::InstallSnapshotRequest,
    ) -> Self::InstallSnapshotFut;
}
