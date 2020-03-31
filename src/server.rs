use crate::AppRaft;
use actix::Addr;
use crate::rpc::Rpc;
use actix_raft::messages;
use crate::Data;
use actix::prelude::Request;
use actix::prelude::Future as ActixFuture;
use futures::Async;

#[derive(Clone)]
struct Server {
    addr: Addr<AppRaft>
}

struct RpcFuture { 
    request: Request<crate::AppRaft, messages::AppendEntriesRequest<Data>>//messages::AppendEntriesRequest<Data>>
}

impl std::future::Future for RpcFuture { 
    type Output = Result<messages::AppendEntriesResponse, ()>;

    fn poll(mut self: std::pin::Pin<&mut Self>, _context: &mut std::task::Context<'_>) -> std::task::Poll<std::result::Result<actix_raft::messages::AppendEntriesResponse, ()>> {
        match ActixFuture::poll(&mut self.request) {
            Ok(Async::NotReady) => std::task::Poll::Pending,//Poll::Ready(obj), //Ok(obj),
            Ok(Async::Ready(val)) => std::task::Poll::Ready(val),//Poll::Ready(obj), //Ok(obj),
            Err(_) => std::task::Poll::Ready(Err(()))
        }
    }
}

impl Rpc for Server {
    type SendClientPayloadFut = RpcFuture;

    fn send_client_payload(self, _: tarpc::context::Context, _msg: messages::AppendEntriesRequest<Data>) -> Self::SendClientPayloadFut {
        unimplemented!()
    }
}
