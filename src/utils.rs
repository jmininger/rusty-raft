// Can only increase if an election starts
pub type TermId = u64;

pub type NodeId = u64;

pub type LogIndex = usize;

pub struct LogEntry {
    pub term: TermId,
    pub message: Vec<u8>,
    pub index: LogIndex,
}

#[derive(Clone, PartialEq)]
pub enum ServerType {
    Follower,
    Candidate,
    Leader,
}
