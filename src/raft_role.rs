use std::collections::HashMap;

use crate::{
    peer::PeerName,
    raft_core::LogIndex,
};

#[derive(Debug)]
pub enum RaftRole {
    /// The server is a follower in the Raft consensus algorithm and contains the current leaders
    /// id
    Follower(PeerName),
    Candidate,
    // Candidate(CandidateState),
    Leader(LeaderState),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LeaderState {
    next_index: HashMap<PeerName, LogIndex>,
    match_index: HashMap<PeerName, LogIndex>, // For each known server:
}

// #[derive(Debug, Default)]
// pub struct CandidateState {
//     votes: HashMap<PeerName, bool>,
//     // We call this timer every time we receive either a response with a termId higher than ours
// or     // if we receive an appendEntries with a termId higher than ours
//     // cancel_trigger: oneshot::Sender<()>,
// }
