use std::sync::Arc;

use crate::{
    raft_state::*,
    rpc::*,
};

pub trait Persist {}

#[derive(Debug, Clone)]
pub struct RaftProtocol {
    pub state: RaftState,
    pub rpc_client: Arc<RpcClient>,
}

// impl<DB: Persist> Context<DB> {
//     pub fn new(state: RaftState, peers: PeerMap, db: DB) -> Self {
//         let rpc_client = Arc::new(RpcClient::new(peers));
//         Self {
//             state,
//             rpc_client,
//             db,
//         }
//     }

//     fn init_election(&mut self) {
//         self.state.election_transition();
//         // self.rpc_client.broadcast("request_vote");
//     }
// }
// pub fn handle_append_entry(&mut self, message: AppendEntry) {
// if !self.is_leader()
//     || message.term < self.current_term
//     ||
// }
