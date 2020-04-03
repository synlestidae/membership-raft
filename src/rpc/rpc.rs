use tarpc;
use actix_raft::messages;
use crate::raft::Transition;

#[tarpc::service]
pub trait Rpc {
    async fn send_client_payload(msg: messages::AppendEntriesRequest<Transition>) -> Result<messages::AppendEntriesResponse, ()>;
}
