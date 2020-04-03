use actix;
use actix_raft;
use log::debug;

/// Your application's metrics interface actor.
pub struct AppMetrics {/* ... snip ... */}

impl actix::Actor for AppMetrics {
    type Context = actix::Context<Self>;

    // ... snip ... other actix methods can be implemented here as needed.
}

impl actix::Handler<actix_raft::RaftMetrics> for AppMetrics {
    type Result = ();

    fn handle(&mut self, msg: actix_raft::RaftMetrics, _ctx: &mut actix::Context<Self>) -> Self::Result {
        debug!("Metrics: {:?}", msg)
    }
}

