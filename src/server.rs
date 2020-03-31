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

struct RpcFuture { 
    //future: Box<dyn Future<Output=std::result::Result<R, E>>>
    request: Request<crate::AppRaft, messages::AppendEntriesRequest<Data>>//messages::AppendEntriesRequest<Data>>
}

//use futures::Async as FuturesAsync;
use actix::prelude::Future as ActixFuture;

use futures::Poll;
use futures::Async;

use std::task::Context;

impl std::future::Future for RpcFuture { 
    type Output = Result<messages::AppendEntriesResponse, ()>;

    fn poll(self: std::pin::Pin<&mut Self>, _context: &mut std::task::Context<'_>) -> std::task::Poll<std::result::Result<actix_raft::messages::AppendEntriesResponse, ()>> {
        match ActixFuture::poll(&mut self.request) {
            Ok(Async::NotReady) => std::task::Poll::Pending,//Poll::Ready(obj), //Ok(obj),
            Ok(Async::Ready(val)) => std::task::Poll::Ready(val),//Poll::Ready(obj), //Ok(obj),
            Err(err) => std::task::Poll::Ready(Err(()))
        }
    }
}

impl Rpc for Server {
    type SendClientPayloadFut = RpcFuture;//<messages::AppendEntriesRequest<Data>>  //Request<crate::AppRaft, >; //RpcFuture<>;

    fn send_client_payload(self, _: tarpc::context::Context,msg: messages::AppendEntriesRequest<Data>) -> Self::SendClientPayloadFut {
        unimplemented!()
    }
}
