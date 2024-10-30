use std::time::Duration;

use crate::{
    json_rpc::{
        RpcRequest,
        RpcResponse,
    },
    network::NetworkManager,
    peer::PeerName,
    raft_core::{
        LogEntry,
        LogIndex,
        Term,
    },
    raft_role::RaftRole,
    raft_rpc::{
        RequestVote,
        RequestVoteResponse,
    },
};

const ELECTION_TIMEOUT: u64 = 5;

#[derive(Debug)]
pub struct RaftState {
    cluster_size: usize,
    host_name: PeerName,

    commit_index: LogIndex,
    last_applied: LogIndex,
    node_type: RaftRole,
    current_term: Term,
    log: Vec<LogEntry>,
    // committed: Log,
}

impl RaftState {
    pub fn new(peer_name: PeerName, cluster_size: usize, leader_name: PeerName) -> Self {
        RaftState {
            cluster_size,
            host_name: peer_name,
            log: vec![],
            commit_index: 0,
            last_applied: 0,
            // All nodes begin as followers, but they may not necessarily know who their follower
            // is
            node_type: RaftRole::Follower(leader_name),
            current_term: 0,
            // log: Log::new(),
            // committed: Log::new(),
        }
    }

    // fn log_index(&self) -> LogIndex {
    //     self.log.len() - 1
    // }

    pub fn init_election(&mut self) -> RequestVote {
        self.node_type = RaftRole::Candidate;
        self.current_term += 1;
        let last_log = self.log.last().cloned().unwrap_or_default();
        RequestVote {
            term: self.current_term,
            candidate_id: self.host_name.clone(),
            last_log_index: last_log.index,
            last_log_term: last_log.term,
        }
    }

    pub fn is_valid_term(&self, term: Term) -> bool {
        term >= self.current_term
    }

    pub fn transition_role(&mut self, role: RaftRole, term: Term) {
        self.current_term = term;
        self.node_type = role;
    }

    pub fn current_term(&self) -> Term {
        self.current_term
    }

    pub fn cluster_size(&self) -> usize {
        self.cluster_size
    }
}

// async fn run_election(network: &mut NetworkManager, raft_state: &mut RaftState) {
//     // In the main loop check for either appends or other candidates and respond accordingly when
// a     // candidate yourself
//     let req_vote = raft_state.init_election();
//     let params = serde_json::to_value(req_vote).unwrap();
//     let rpc_req = RpcRequest {
//         id: network.next_request_id().into(),
//         method: "request_vote".to_string(),
//         params,
//     };
//     // let election_timeout = Duration::from_secs(ELECTION_TIMEOUT);
//     let rpc_timeout = Duration::from_millis(250);
//     let mut response_stream = network.broadcast(rpc_req, rpc_timeout);

//     let mut votes = 0;
//     while let Some(response) = response_stream.join_next().await {
//         let (_peer_name, result) = response.unwrap_or_default();
//         if let Some(RpcResponse {
//             result: resp_raw, ..
//         }) = result
//         {
//             if let Ok(RequestVoteResponse { term, vote_granted }) =
//                 serde_json::from_value::<RequestVoteResponse>(resp_raw)
//             {
//                 if term > raft_state.current_term {
//                     raft_state.current_term = term;
//                     raft_state.node_type = RaftRole::Follower(Default::default());
//                     return;
//                 }
//                 if vote_granted {
//                     votes += 1;
//                     if votes > raft_state.cluster_size / 2 {
//                         raft_state.node_type = RaftRole::Leader(Default::default());
//                         return;
//                     }
//                 }
//             }
//         }
//     }
// }
