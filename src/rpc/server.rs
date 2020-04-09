/*use crate::AppRaft;
use actix::Addr;
use crate::rpc::Rpc;
use actix_raft::messages;
use actix::Message;
use actix::prelude::Request;
use actix::Handler;
use actix::dev::ToEnvelope;
use crate::raft::Transition;

#[derive(Clone)]
pub struct Server {
    addr: Addr<AppRaft>
}

#[allow(dead_code)]
pub struct RpcFuture<A, M: Message + std::marker::Send> where
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

    fn poll(self: std::pin::Pin<&mut Self>, _context: &mut std::task::Context<'_>) -> std::task::Poll<M::Result> {
        unimplemented!()
    }
}

impl Rpc for Server {
    type SendClientPayloadFut = RpcFuture<AppRaft, messages::AppendEntriesRequest<Transition>>;

    fn send_client_payload(self, _: tarpc::context::Context, msg: messages::AppendEntriesRequest<Transition>) -> Self::SendClientPayloadFut {
        RpcFuture { request: self.addr.send(msg) }
    }
}*/
