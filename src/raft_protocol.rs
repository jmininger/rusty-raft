use std::time::Duration;

use tokio::{
    net::TcpStream,
    sync::mpsc::{
        Receiver,
        Sender,
    },
};

use crate::{
    json_rpc::{
        RpcRequest,
        RpcResponse,
    },
    network::{
        handle_new_inbound_connection,
        NetworkManager,
    },
    peer::PeerId,
    raft_role::RaftRole,
    raft_rpc::{
        AppendEntry,
        RequestVote,
        RequestVoteResponse,
    },
    raft_state::RaftState,
};

const RAFT_APPEND_ENTRIES: &str = "append_entries";

const RAFT_REQUEST_VOTE: &str = "request_vote";

pub struct EventListeners {
    new_conn_alert: Receiver<TcpStream>,
    req_alert: Receiver<(PeerId, (RpcRequest, Sender<RpcResponse>))>,
}

async fn run_candidate_loop(
    event_listeners: &mut EventListeners,
    network: &mut NetworkManager,
    raft_state: &mut RaftState,
    hid: PeerId,
) {
    let req_vote = raft_state.init_election();
    let params = serde_json::to_value(req_vote).unwrap();
    let rpc_req = RpcRequest {
        id: network.next_request_id().into(),
        method: RAFT_REQUEST_VOTE.to_string(),
        params,
    };
    // let election_timeout = Duration::from_secs(ELECTION_TIMEOUT);
    let rpc_timeout = Duration::from_millis(250);
    let mut response_stream = network.broadcast(rpc_req, rpc_timeout);
    // Start with one vote for self
    let mut votes = 1;
    tokio::select! {
        Some(raw_sock) = event_listeners.new_conn_alert.recv() => {
            let hid = hid.clone();
            // todo: refactor code so we aren't awaiting on identity-exchange
            // Thought I cant do this without locking the network manager
            handle_new_inbound_connection(network, hid, raw_sock).await;
        },
        Some((pid, (RpcRequest { id, method, params }, resp_trigger))) = event_listeners.req_alert.recv() => {
            match method.as_str() {
                RAFT_APPEND_ENTRIES => {
                    let req = serde_json::from_value::<AppendEntry>(params).map_err(|e| {
                        tracing::error!("Error deserializing append entry: {}", e);
                    }).expect("Error deserializing append entry");
                    if raft_state.is_valid_term(req.term) {
                        raft_state.transition_role(RaftRole::Follower(pid.common_name), req.term);
                        //todo
                        return;
                    }

                },
                RAFT_REQUEST_VOTE => {
                    let req = serde_json::from_value::<RequestVote>(params).map_err(|e| {
                        tracing::error!("Error deserializing request vote: {}", e);
                    }).expect("Error deserializing request vote");
                    if raft_state.is_valid_term(req.term) {
                        raft_state.transition_role(RaftRole::Follower(pid.common_name), req.term);
                        //todo
                        return;
                    } else {
                        let vote_granted = false;
                        // if req.last_log_index >= raft_state.log_index() {
                        //     vote_granted = true;
                        // }
                        let resp = RpcResponse {
                            id,
                            result: serde_json::to_value(RequestVoteResponse {
                                term: raft_state.current_term(),
                                vote_granted,
                            }).expect("Error serializing request vote response"),
                        };
                        resp_trigger.try_send(resp).expect("Error sending response");
                    }

                },
                _ => {
                    tracing::warn!("Unknown method: {} received during candidate loop", method);
                },
            }

        },
        Some(ev) = response_stream.join_next() =>  {
            let (_peer_name, result) = ev.unwrap_or_default();
            if let Some(RpcResponse {
                result: resp_raw, ..
            }) = result
            {
                if let Ok(RequestVoteResponse { term, vote_granted }) =
                    serde_json::from_value::<RequestVoteResponse>(resp_raw)
                {
                    // Todo: Double check this logic
                    if !raft_state.is_valid_term(term) {
                        raft_state.transition_role(RaftRole::Follower(Default::default()), term);
                        return;
                    }
                    if vote_granted {
                        votes += 1;
                    }
                    if votes > raft_state.cluster_size() / 2 {
                        raft_state.transition_role(RaftRole::Leader(Default::default()), raft_state.current_term());
                        return;
                    }
                }
            }
        },

    }
}
