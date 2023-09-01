use crate::utils::*;

pub struct AppendEntry {
    pub term: TermId,
    pub leader_id: u64,
    pub prev_log_index: LogIndex,
    pub prev_log_term: TermId,
    pub entries: Vec<LogEntry>,
    pub leader_commit: LogIndex,
}

pub struct AppendEntryResponse {
    pub term: TermId,
    pub success: bool,
}

// Note we need to have an rpc for normal clients to add messages to the leader;
