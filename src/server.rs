use crate::AppRaft;
use actix::Addr;
use crate::rpc::Rpc;
use actix_raft::messages;
use crate::Data;
use actix::Message;
use actix::prelude::Request;
use actix::prelude::Future as ActixFuture;
use futures::Async;
use actix::Handler;
use actix::dev::ToEnvelope;

#[derive(Clone)]
struct Server {
    addr: Addr<AppRaft>
}

struct RpcFuture<A, M: Message + std::marker::Send> where 
    <M as actix::Message>::Result: std::marker::Send,
    A: Handler<M>,
    A::Context: ToEnvelope<A, M>,
{ 
    request: Request<A, M>
}

impl<A, M: Message + std::marker::Send> std::future::Future for RpcFuture<A, M> where 
    <M as actix::Message>::Result: Send,
    A::Context: ToEnvelope<A, M>,
    A: Handler<M> {
    type Output = M::Result;

    fn poll(mut self: std::pin::Pin<&mut Self>, _context: &mut std::task::Context<'_>) -> std::task::Poll<M::Result> {
        let this = &mut self;
        match ActixFuture::poll(&mut this.request) {
            Ok(Async::NotReady) => std::task::Poll::Pending,//Poll::Ready(obj), //Ok(obj),
            Ok(Async::Ready(val)) => std::task::Poll::Ready(val),//Poll::Ready(obj), //Ok(obj),
            Err(err) => unimplemented!() //std::task::Poll::Ready(err)
        }
    }
}

impl Rpc for Server {
    type SendClientPayloadFut = RpcFuture<AppRaft, messages::AppendEntriesRequest<Data>>;

    fn send_client_payload(self, _: tarpc::context::Context, msg: messages::AppendEntriesRequest<Data>) -> Self::SendClientPayloadFut {
        self.addr.send(msg);
        unimplemented!()
    }
}
