use crate::AppRaft;
use actix::Addr;
use crate::rpc::Rpc;
use actix_raft::messages;
use crate::Data;
use futures::future::Future;
use core::pin::Pin;
use actix::prelude::Request;

#[derive(Clone)]
struct Server {
    addr: Addr<AppRaft>
}

struct RpcFuture<R, E> { 
    future: Box<dyn Future<Output=std::result::Result<R, E>>>
}

impl<R, E> futures::future::Future for RpcFuture<R, E> { 
    type Output = Result<R, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut futures::task::Context) -> futures::task::Poll<Self::Output> {
        unimplemented!()
    }
}

impl Rpc for Server {
    type SendClientPayloadFut = Request<crate::AppRaft, messages::AppendEntriesRequest<Data>>; //RpcFuture<>;

    fn send_client_payload(self, _: tarpc::context::Context,msg: messages::AppendEntriesRequest<Data>) -> Self::SendClientPayloadFut {
        unimplemented!()
    }
}
