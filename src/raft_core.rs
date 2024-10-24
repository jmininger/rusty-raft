use serde::{
    Deserialize,
    Serialize,
};

pub type LogIndex = usize;

pub type Term = u64;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogEntry {
    pub term: Term,
    pub message: String,
    pub index: LogIndex,
}
