use crate::node::AppNode;
use crate::rpc::{CreateSessionRequest, CreateSessionResponse};
use crate::rpc::{RpcClient, RpcError};
use actix_raft::messages;
use futures::future::Future;
use reqwest::r#async::Client;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{to_vec};
use log::debug;

#[derive(Clone)]
pub struct HttpRpcClient {
    client: Client,
}

impl HttpRpcClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn post<'r, T: DeserializeOwned + 'static>(
        &self,
        url: &Url,
        request: &RpcRequest,
    ) -> HttpFut<T> {
        Box::new(
            self.client
                .post(url.clone())
                .json(request)
                .send()
                .and_then(|mut res| res.json())
                .map_err(|err| RpcError::from(err)),
        )
    }
}

pub type HttpFut<T> = Box<dyn Future<Item = T, Error = RpcError>>;

impl RpcClient for HttpRpcClient {
    type JoinClusterFut = HttpFut<CreateSessionResponse>;
    type GetNodesFut = HttpFut<Vec<AppNode>>;
    type AppendEntriesFut = HttpFut<messages::AppendEntriesResponse>;
    type VoteFut = HttpFut<messages::VoteResponse>;
    type InstallSnapshotFut = HttpFut<messages::InstallSnapshotResponse>;

    fn join_cluster(&self, url: &Url, msg: CreateSessionRequest) -> Self::JoinClusterFut {
        debug!("Joining cluster at {}", url);

        self.post(url, &RpcRequest::JoinCluster(msg.clone()))
    }

    fn get_nodes(&self, url: &Url) -> Self::GetNodesFut {
        debug!("Getting nodes from {}", url);

        self.post(url, &RpcRequest::GetNodes)
    }

    fn append_entries(
        &self,
        url: &Url,
        msg: messages::AppendEntriesRequest<crate::raft::Transition>,
    ) -> Self::AppendEntriesFut {
        debug!("Appending entries to {}", url);

        self.post(url, &RpcRequest::AppendEntries(msg))
    }

    fn vote(&self, url: &Url, msg: messages::VoteRequest) -> Self::VoteFut {
        debug!("Voting at {}", url);

        self.post(url, &RpcRequest::Vote(msg))
    }

    fn install_snapshot(
        &self,
        url: &Url,
        msg: messages::InstallSnapshotRequest,
    ) -> Self::InstallSnapshotFut {
        debug!("Installing snapshot at {}", url);

        self.post(url, &RpcRequest::InstallSnapshot(msg))
    }
}

#[derive(Serialize, Deserialize)]
pub enum RpcRequest {
    JoinCluster(CreateSessionRequest),
    GetNodes,
    AppendEntries(messages::AppendEntriesRequest<crate::raft::Transition>),
    Vote(messages::VoteRequest),
    InstallSnapshot(messages::InstallSnapshotRequest),
}
