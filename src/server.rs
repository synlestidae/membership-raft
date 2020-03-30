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

use crate::actix::prelude::Future as ActixFuture;

//use futures::Async as FuturesAsync;
use futures::task::Poll as FuturesPoll;

impl futures::future::Future for RpcFuture { 
    type Output = Result<messages::AppendEntriesResponse, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut futures::task::Context) -> futures::task::Poll<Self::Output> {
        //match ActixFuture::poll(&mut self.request) {
            //Ok(FuturesPoll::Pending) => std::task::Poll::Pending,
            //Ok(FuturesPoll::Ready(r)) => std::task::Poll::Ready(r),
        //    Err(err) => unimplemented!(),
        //    _ => unimplemented!(),
        //}
        unimplemented!()

        //self.request.poll()
        //unimplemented!()
    }
}

impl Rpc for Server {
    type SendClientPayloadFut = RpcFuture;//<messages::AppendEntriesRequest<Data>>  //Request<crate::AppRaft, >; //RpcFuture<>;

    fn send_client_payload(self, _: tarpc::context::Context,msg: messages::AppendEntriesRequest<Data>) -> Self::SendClientPayloadFut {
        unimplemented!()
    }
}
