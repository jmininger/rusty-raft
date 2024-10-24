use std::collections::HashMap;

use crate::peer::PeerName;

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
    current_term: Term,
    // log: Log,
    // committed: Log,
}

pub type LogEntry = String;

pub type LogIndex = usize;

pub type Term = u64;
