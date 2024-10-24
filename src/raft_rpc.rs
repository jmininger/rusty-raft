use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    peer::PeerName,
    raft_state::{
        LogEntry,
        LogIndex,
        Term,
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub struct AppendEntry {
    pub term: Term,
    pub leader_id: u64,
    pub prev_log_index: LogIndex,
    pub prev_log_term: Term,
    pub entries: Vec<LogEntry>,
    pub leader_commit: LogIndex,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppendEntryResponse {
    pub term: Term,
    pub success: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestVote {
    pub term: Term,
    pub candidate_id: PeerName,
    pub last_log_index: LogIndex,
    pub last_log_term: Term,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    pub term: Term,
    pub vote_granted: bool,
}
