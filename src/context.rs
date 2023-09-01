use crate::messages::*;
use crate::raft_state::*;
use crate::utils::*;

pub struct ServerContext {
    db: Box<std::fs::File>,
    state: RaftState,
    // peers: Vec<Peer>,
}

impl ServerContext {
    pub fn handle_append_entry(&mut self, message: AppendEntry) {
        // if !self.is_leader()
        //     || message.term < self.current_term
        //     ||
    }
}
