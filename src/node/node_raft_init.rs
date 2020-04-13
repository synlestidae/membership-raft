use crate::raft::RaftFut;
use crate::AppRaft;
use actix;

pub struct NodeRaftInit {
    addr: actix::Addr<AppRaft>,
}

impl actix::Message for NodeRaftInit {
    type Result = RaftFut<(), ()>;
}
