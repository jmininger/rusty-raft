use std::collections::HashMap;

use crate::{
    log::*,
    rpc::*,
    utils::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderState {
    next_index: HashMap<NodeId, LogIndex>,
    match_index: HashMap<NodeId, LogIndex>, // For each known server:
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerType {
    Follower,
    Candidate,
    Leader(LeaderState),
}

#[derive(Debug, Clone)]
pub struct RaftState {
    commit_index: LogIndex,
    last_applied: LogIndex,
    node_type: ServerType,
    current_term: TermId,
    log: Log,
    // committed: Log,
}

impl RaftState {
    pub fn new(is_leader: bool) -> Self {
        let node_type = match is_leader {
            false => ServerType::Follower,
            true => ServerType::Leader(LeaderState {
                next_index: HashMap::new(),
                match_index: HashMap::new(),
            }),
        };
        Self {
            commit_index: 0,
            last_applied: 0,
            node_type,
            current_term: 0,
            log: Log::new(),
        }
    }

    pub fn is_leader(&self) -> bool {
        match &self.node_type {
            ServerType::Leader(_) => true,
            _ => false,
        }
    }

    pub fn commit_entry(&mut self, entry: LogEntry) {
        // Also commits all prior entries in the log;
        todo!();
    }

    pub fn election_transition(&mut self) {
        self.node_type = ServerType::Candidate;
        self.current_term += 1;
    }

    // Should be applied to all requests AND responses received
    // pub fn verify_term(&self, term: TermId) {
    //     if term > self.current_term {
    //         self.node_type = ServerType::Follower;
    //         self.current_term = term;
    //     }
    // }

    pub fn handle_append(&mut self, message: AppendEntry) -> AppendEntryResponse {
        if message.term < self.current_term
            || self
                .log
                .invalid_term_bounds(message.prev_log_index, message.prev_log_term)
        {
            AppendEntryResponse {
                term: self.current_term,
                success: false,
            }
        } else {
            // if self.is_leader() {
            //     // Need to potentially remove ourselves as a leader
            //     todo!()
            // }

            self.log.append_entries(message.entries);

            if message.leader_commit > self.commit_index {
                self.set_commit_index(message.leader_commit);
            }
            AppendEntryResponse {
                term: self.current_term,
                success: true,
            }
        }
    }

    fn set_commit_index(&mut self, leader: LogIndex) {
        let most_recent_index = self.log.get_last_index();
        self.commit_index = std::cmp::min(leader, most_recent_index);
    }
}

//Represents a change to the state, but not necessarily the persisted state itself
// struct PersistEntry {
//     current_term: u64, // must have ordering relative to each other
//     voted_for: Option<u64>,
//     log: Vec<LogEntry>,
// }
