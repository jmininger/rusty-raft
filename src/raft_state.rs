use std::collections::HashMap;

use crate::rpc::{
    LogEntry,
    LogIndex,
    PeerName,
    TermId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderState {
    next_index: HashMap<PeerName, LogIndex>,
    match_index: HashMap<PeerName, LogIndex>, // For each known server:
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
