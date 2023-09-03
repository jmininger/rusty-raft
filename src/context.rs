use crate::peer::*;
use crate::raft_state::*;
use crate::rpc::*;

use std::sync::Arc;

pub struct ServerContext {
    db: Box<std::fs::File>,
    state: RaftState,
    rpc_client: Arc<RpcClient>,
    peers: PeerMap,
}

impl ServerContext {
    pub fn handle_append_entry(&mut self, message: AppendEntry) {
        // if !self.is_leader()
        //     || message.term < self.current_term
        //     ||
    }
}
