use crate::futures::Stream;
use crate::node::AppNode;
use crate::rpc::{HttpRpcClient, RpcClient, RpcError};
use reqwest::Url;

pub struct RaftDiscovery {
    hosts: Vec<Url>,
    rpc_client: HttpRpcClient,
}

impl RaftDiscovery {
    pub fn new(hosts: Vec<Url>, rpc_client: HttpRpcClient) -> Self {
        Self { hosts, rpc_client }
    }
}

pub struct DiscoveredNodes {
    pub host: String,
    pub nodes: Vec<AppNode>,
}

#[derive(Debug)]
pub enum DiscoveryErr {
    NoHosts,
    RpcErrors(Vec<RpcError>),
}

pub struct DiscoveryFuture {
    stream: Box<dyn Stream<Item = Vec<AppNode>, Error = RpcError>>,
    errors: Vec<RpcError>,
}
use futures::prelude::Async;

impl futures::future::Future for DiscoveryFuture {
    type Item = Vec<AppNode>;
    type Error = DiscoveryErr;

    fn poll(&mut self) -> Result<futures::Async<Self::Item>, Self::Error> {
        match self.stream.poll() {
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(None)) => {
                Err(DiscoveryErr::RpcErrors(self.errors.drain(0..).collect()))
            }
            Ok(Async::Ready(Some(ready))) => Ok(Async::Ready(ready)),
            Err(err) => {
                self.errors.push(err);

                return Ok(Async::NotReady);
            }
        }
    }
}

impl RaftDiscovery {
    pub fn nodes(&self) -> DiscoveryFuture {
        let client = self.rpc_client.clone();

        let stream = futures::stream::iter_ok(self.hosts.clone().into_iter())
            .and_then(move |url| client.get_nodes(&url));

        DiscoveryFuture {
            stream: Box::new(stream),
            errors: vec![],
        }
    }
}
