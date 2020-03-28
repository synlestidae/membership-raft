use actix_raft::metrics;
use actix::Actor;
use actix::Context;
use actix::Handler;

pub struct AppMetrics {} 

impl AppMetrics {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for AppMetrics {
    type Context = Context<Self>;
}

impl Handler<metrics::RaftMetrics> for AppMetrics {
    type Result = ();//Result<(), ()>;

    fn handle(&mut self, msg: metrics::RaftMetrics, ctx: &mut Self::Context) -> Self::Result {
    }
}

