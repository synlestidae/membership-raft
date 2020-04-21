use actix::dev::MessageResponse;
use actix::dev::ResponseChannel;
use futures::future::Future;
use futures::Async;

pub struct RaftFut<T: Send, E: Send>(pub Box<dyn Future<Item = T, Error = E>>);

impl<T: Send, E: Send> Future for RaftFut<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.0.poll()
    }
}

impl<T: Send, E: Send> RaftFut<T, E> {
    pub fn from<F: Future<Item = T, Error = E> + 'static>(fut: F) -> RaftFut<T, E> {
        RaftFut(Box::new(fut))
    }
}

impl<T: Send, E: Send, A: actix::Actor, M: actix::Message> MessageResponse<A, M> for RaftFut<T, E> {
    fn handle<R: ResponseChannel<M>>(self, _ctx: &mut A::Context, _tx: Option<R>) {}
}
