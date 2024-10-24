use std::collections::HashMap;

use crate::{
    network::NetworkManager,
    peer::PeerName,
    raft_core::{
        LogEntry,
        LogIndex,
        Term,
    },
    raft_rpc::RequestVote,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderState {
    next_index: HashMap<PeerName, LogIndex>,
    match_index: HashMap<PeerName, LogIndex>, // For each known server:
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Follower,
    Candidate,
    Leader(LeaderState),
}

#[derive(Debug, Clone)]
pub struct RaftState {
    peer_name: PeerName,
    commit_index: LogIndex,
    last_applied: LogIndex,
    node_type: ServerState,
    current_term: Term,
    log: Vec<LogEntry>,
    // committed: Log,
}

impl RaftState {
    pub fn new(peer_name: PeerName) -> Self {
        RaftState {
            peer_name,
            log: vec![],
            commit_index: 0,
            last_applied: 0,
            node_type: ServerState::Follower,
            current_term: 0,
            // log: Log::new(),
            // committed: Log::new(),
        }
    }

    pub fn begin_term(&mut self, term: Term) {
        self.current_term = term;
        self.node_type = ServerState::Follower;
    }

    fn log_index(&self) -> LogIndex {
        self.log.len() - 1
    }

    pub fn initiate_election(&mut self, network: &mut NetworkManager) {
        self.node_type = ServerState::Candidate;
        self.current_term += 1;
        let last_log = self.log.last().cloned().unwrap_or_default();
        let rpc_req = RequestVote {
            term: self.current_term,
            candidate_id: self.peer_name.clone(),
            last_log_index: last_log.index,
            last_log_term: last_log.term,
        };
    }
}
