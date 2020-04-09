use crate::error::Error;
use crate::node::SharedNetworkState;
use crate::raft::DataResponse;
use crate::raft::Transition;
use actix::fut::result;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages;
use actix_raft::storage;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Serialize, Deserialize)]
pub struct AppStorage {
    shared_network_state: SharedNetworkState,
    snapshot_path: Option<String>,
    membership: messages::MembershipConfig,
    logs: Vec<messages::Entry<Transition>>,
}

impl AppStorage {
    pub fn new(
        shared_network_state: SharedNetworkState,
        membership: messages::MembershipConfig,
    ) -> Self {
        Self {
            shared_network_state,
            snapshot_path: None,
            membership,
            logs: vec![],
        }
    }

    fn upsert_entry(&mut self, entry: messages::Entry<Transition>) -> Result<(), Error> {
        for (i, e) in self.logs.iter_mut().enumerate() {
            if i as u64 == e.index {
                mem::replace(e, entry);
                return Ok(());
            }
        }

        self.logs.push(entry);
        Ok(())

        /*if self.logs.len() == entry.index as usize {
        } else {
            Ok(())
        }*/
    }

    fn apply_to_state_machine(&mut self, data: Transition) {
        debug!("Received message: {:?}", data);
        match data {
            Transition::AddNode {
                id,
                name,
                host,
                port,
            } => {
                self.shared_network_state
                    .register_node(id, name, host, port);
            }
        }
    }

    fn apply_entry_to_state_machine(&mut self, msg: messages::EntryPayload<Transition>) {
        trace!("Applying msg {:?}", msg);
        match msg {
            messages::EntryPayload::Blank => {}
            messages::EntryPayload::Normal(messages::EntryNormal { data }) => {
                self.apply_to_state_machine(data);
            }
            messages::EntryPayload::ConfigChange(messages::EntryConfigChange { membership }) => {
                self.membership = membership;
            }
            messages::EntryPayload::SnapshotPointer(messages::EntrySnapshotPointer { path }) => {
                self.snapshot_path = Some(path);
            }
        }
    }
}

impl Actor for AppStorage {
    type Context = Context<Self>;
}

impl storage::RaftStorage<Transition, DataResponse, Error> for AppStorage {
    type Actor = Self;

    type Context = Context<Self>;
}

impl Handler<storage::GetInitialState<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, storage::InitialState, Error>;

    fn handle(
        &mut self,
        _msg: storage::GetInitialState<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling GetInitialState");
        debug!("Members in initial state: {:?}", self.membership);

        Box::new(result(Ok(storage::InitialState {
            last_log_index: 0,
            last_log_term: 0,
            last_applied_log: 0,
            hard_state: storage::HardState {
                current_term: 0,
                voted_for: None,
                membership: self.membership.clone(),
            },
        })))
    }
}

impl Handler<storage::SaveHardState<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::SaveHardState<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling SaveHardState");

        Box::new(result(Ok(())))
    }
}

impl Handler<storage::GetLogEntries<Transition, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, Vec<messages::Entry<Transition>>, Error>;

    fn handle(
        &mut self,
        msg: storage::GetLogEntries<Transition, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling GetLogEntries: {}, {}", msg.start, msg.stop);

        let r = self
            .logs
            .iter()
            .skip(msg.start as usize)
            .take(msg.stop as usize - msg.start as usize)
            .cloned()
            .collect::<Vec<_>>();

        info!("GetLogEntries: Got {} logs: {:?}", r.len(), r);

        Box::new(result(Ok(r)))
    }
}

impl Handler<storage::AppendEntryToLog<Transition, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        msg: storage::AppendEntryToLog<Transition, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling AppendEntryToLog");

        let new_msg = (*msg.entry).clone();

        Box::new(result(self.upsert_entry(new_msg)))
    }
}

impl Handler<storage::ReplicateToLog<Transition, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        msg: storage::ReplicateToLog<Transition, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling ReplicateToLog");

        for entry in (*msg.entries).clone().into_iter() {
            match self.upsert_entry(entry) {
                Ok(_) => {}
                err => return Box::new(result(err)),
            }
        }

        Box::new(result(Ok(())))
    }
}

impl Handler<storage::ApplyEntryToStateMachine<Transition, DataResponse, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, DataResponse, Error>;

    fn handle(
        &mut self,
        msg: storage::ApplyEntryToStateMachine<Transition, DataResponse, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling ApplyEntryToStateMachine");

        let payload = msg.payload.payload.clone();

        self.apply_entry_to_state_machine(payload);

        Box::new(result(Ok(DataResponse::success(
            "Successfully applied entry to state machine",
        ))))
    }
}

impl Handler<storage::ReplicateToStateMachine<Transition, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        msg: storage::ReplicateToStateMachine<Transition, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling ReplicateToStateMachine: {:?}", msg.payload);

        for e in msg.payload {
            self.apply_entry_to_state_machine(e.payload);
        }

        // ... snip ...
        Box::new(result(Ok(())))
    }
}

impl Handler<storage::CreateSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, storage::CurrentSnapshotData, Error>;

    fn handle(
        &mut self,
        _msg: storage::CreateSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling CreateSnapshot");

        let snapshot_path = self.snapshot_path.clone();
        let latest_log = &self.logs[self.logs.len() - 1];
        let snapshot = storage::CurrentSnapshotData {
            term: latest_log.term,
            index: latest_log.index,
            membership: self.membership.clone(),
            pointer: messages::EntrySnapshotPointer {
                path: match snapshot_path {
                    Some(s) => s.clone(),
                    None => String::from("."),
                },
            },
        };

        Box::new(result(Ok(snapshot)))
    }
}

impl Handler<storage::InstallSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        _msg: storage::InstallSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling InstallSnapshot");

        Box::new(result(Ok(())))
    }
}

impl Handler<storage::GetCurrentSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, Option<storage::CurrentSnapshotData>, Error>;

    fn handle(
        &mut self,
        _msg: storage::GetCurrentSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("Handling GetCurrentSnapshot");

        Box::new(result(Ok(None)))
    }
}
