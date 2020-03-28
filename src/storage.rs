use crate::error::Error;
use crate::Data;
use crate::DataResponse;
use actix::fut::result;
use actix::{Actor, Context, Handler, ResponseActFuture};
use actix_raft::messages;
use actix_raft::storage;
use std::mem;

pub struct AppStorage {
    snapshot_path: Option<String>,
    membership: messages::MembershipConfig,
    logs: Vec<messages::Entry<Data>>,
}

impl AppStorage {
    fn upsert_entry(&mut self, entry: messages::Entry<Data>) -> Result<(), Error> {
        for (i, e) in self.logs.iter_mut().enumerate() {
            if i as u64 == e.index {
                mem::replace(e, entry);
                return Ok(());
            }
        }

        if self.logs.len() == entry.index as usize {
            self.logs.push(entry);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn apply_to_state_machine(&mut self, _data: Data) {
    }

    fn apply_entry_to_state_machine(&mut self, msg: messages::EntryPayload<Data>) {
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

impl storage::RaftStorage<Data, DataResponse, Error> for AppStorage {
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
        Box::new(result(Ok(storage::InitialState {
            last_log_index: 0,
            last_log_term: 0,
            last_applied_log: 0,
            hard_state: storage::HardState {
                current_term: 0,
                voted_for: None,
                membership: messages::MembershipConfig {
                    is_in_joint_consensus: false,
                    members: vec![],
                    non_voters: vec![],
                    removing: vec![],
                },
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
        Box::new(result(Ok(())))
    }
}

impl Handler<storage::GetLogEntries<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, Vec<messages::Entry<Data>>, Error>;

    fn handle(
        &mut self,
        msg: storage::GetLogEntries<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Box::new(result(Ok(self
            .logs
            .iter()
            .skip(msg.start as usize)
            .take(msg.stop as usize - msg.start as usize)
            .cloned()
            .collect())))
    }
}

impl Handler<storage::AppendEntryToLog<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        msg: storage::AppendEntryToLog<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let new_msg = (*msg.entry).clone();

        Box::new(result(self.upsert_entry(new_msg)))
    }
}

impl Handler<storage::ReplicateToLog<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        msg: storage::ReplicateToLog<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        for entry in (*msg.entries).clone().into_iter() {
            match self.upsert_entry(entry) {
                Ok(_) => {}
                err => return Box::new(result(err)),
            }
        }

        Box::new(result(Ok(())))
    }
}

impl Handler<storage::ApplyEntryToStateMachine<Data, DataResponse, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, DataResponse, Error>;

    fn handle(
        &mut self,
        msg: storage::ApplyEntryToStateMachine<Data, DataResponse, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let payload = msg.payload.payload.clone();

        self.apply_entry_to_state_machine(payload);
        /*match payload {
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
        }*/

        Box::new(result(Ok(DataResponse::success("Successfully applied entry to state machine"))))
    }
}

impl Handler<storage::ReplicateToStateMachine<Data, Error>> for AppStorage {
    type Result = ResponseActFuture<Self, (), Error>;

    fn handle(
        &mut self,
        msg: storage::ReplicateToStateMachine<Data, Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        for e in msg.payload {
            self.apply_entry_to_state_machine(e.payload);
        }

        // ... snip ...
        unimplemented!()
    }
}

impl Handler<storage::CreateSnapshot<Error>> for AppStorage {
    type Result = ResponseActFuture<Self, storage::CurrentSnapshotData, Error>;

    fn handle(
        &mut self,
        _msg: storage::CreateSnapshot<Error>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let snapshot_path = self.snapshot_path.clone();
        let latest_log = &self.logs[self.logs.len() - 1];
        let snapshot = storage::CurrentSnapshotData {
            term: latest_log.term,
            index: latest_log.index,
            membership: self.membership.clone(),
            pointer: messages::EntrySnapshotPointer { 
                path: match snapshot_path { 
                    Some(s) => s.clone(),
                    None => unimplemented!() 
                }
            }
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
        Box::new(result(Ok(None)))
    }
}
