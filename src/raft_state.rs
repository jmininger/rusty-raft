use crate::log::*;
use crate::messages::*;
use crate::utils::*;
use std::collections::HashMap;

pub struct LeaderState {
    next_index: HashMap<NodeId, LogIndex>,
    match_index: HashMap<NodeId, LogIndex>, // For each known server:
}

pub struct RaftState {
    commit_index: LogIndex,
    last_applied: LogIndex,
    leader_state: Option<LeaderState>,
    current_state: ServerType,
    current_term: TermId,
    log: Log,
}

impl RaftState {
    fn new() -> Self {
        RaftState {
            commit_index: 0,
            last_applied: 0,
            leader_state: None,
            current_state: ServerType::Follower,
            current_term: 0,
            log: Log::new(),
        }
    }

    fn is_leader(&self) -> bool {
        self.current_state == ServerType::Leader
    }

    fn commit_entry(&mut self, entry: LogEntry) {
        // Also commits all prior entries in the log;
        todo!();
    }

    fn init_election(&self) {
        todo!();
    }

    fn handle_append(&mut self, message: AppendEntry) {
        if !self.is_leader() {
            // Need to potentially remove ourselves as a leader
            todo!()
        }

        if message.term > self.current_term {
            todo!()
        }
        if message.term < self.current_term {
            // reply false
            todo!()
        }
        let log_term = self.log.retrieve_term(message.prev_log_index);
        if log_term != message.prev_log_term {
            // reply false
            todo!()
        }
        message.entries.into_iter().for_each(|entry| {
            self.log.add_entry(entry);
        });

        if message.leader_commit > self.commit_index {
            todo!()
        }
    }
}

//Represents a change to the state, but not necessarily the persisted state itself
// struct PersistEntry {
//     current_term: u64, // must have ordering relative to each other
//     voted_for: Option<u64>,
//     log: Vec<LogEntry>,
// }
