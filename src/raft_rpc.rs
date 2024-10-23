use serde::{
    Deserialize,
    Serialize,
};

pub type LogEntry = String;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppendEntry {
    pub term: TermId,
    pub leader_id: u64,
    pub prev_log_index: LogIndex,
    pub prev_log_term: TermId,
    pub entries: Vec<LogEntry>,
    pub leader_commit: LogIndex,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppendEntryResponse {
    pub term: TermId,
    pub success: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestVote {
    pub term: TermId,
    pub candidate_id: NodeId,
    pub last_log_index: LogIndex,
    pub last_log_term: TermId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    pub term: TermId,
    pub vote_granted: bool,
}
