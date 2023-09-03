use crate::log::*;
use crate::rpc::*;
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
    node_type: ServerType,
    current_term: TermId,
    log: Log,
}

impl RaftState {
    pub fn new(is_leader: bool) -> Self {
        let (node_type, leader_state) = match is_leader {
            false => (ServerType::Follower, None),
            true => (
                ServerType::Leader,
                Some(LeaderState {
                    next_index: HashMap::new(),
                    match_index: HashMap::new(),
                }),
            ),
        };
        Self {
            commit_index: 0,
            last_applied: 0,
            leader_state,
            node_type,
            current_term: 0,
            log: Log::new(),
        }
    }

    pub fn is_leader(&self) -> bool {
        self.node_type == ServerType::Leader
    }

    pub fn commit_entry(&mut self, entry: LogEntry) {
        // Also commits all prior entries in the log;
        todo!();
    }

    pub fn init_election(&self) {
        todo!();
    }

    pub fn handle_append(&mut self, message: AppendEntry) {
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
