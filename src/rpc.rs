use crate::context;
use crate::peer::*;
use crate::utils::*;

use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
    Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Alias makes it easy to swap things out for now
type AppState = context::RaftProtocol;
// type AppState = ();

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

// Note we need to have an rpc for normal clients to add messages to the leader;
#[derive(Debug, Clone)]
pub struct RpcClient {
    client: Client,
    peer_map: PeerMap,
}

impl RpcClient {
    pub fn new(peer_map: PeerMap) -> Self {
        Self {
            peer_map,
            client: Client::new(),
        }
    }

    async fn request_vote(
        &self,
        target: NodeId,
        message: RequestVote,
    ) -> Result<RequestVoteResponse, reqwest::Error> {
        let peer_addr = self.peer_map.fetch_address(target);
        let resp = self.client.post(peer_addr).json(&message).send().await?;
        resp.json::<RequestVoteResponse>().await
    }

    // send a RequestVote request to the target node

    pub async fn append_entries(
        &self,
        target: NodeId,
        message: AppendEntry,
    ) -> Result<AppendEntryResponse, reqwest::Error> {
        let peer_addr = self.peer_map.fetch_address(target);
        let resp = self.client.post(peer_addr).json(&message).send().await?;
        resp.json::<AppendEntryResponse>().await
    }
}

pub async fn run_rpc_server(app_state: AppState, port: u16) {
    let app = Router::new().nest(
        "/consensus",
        Router::new()
            .route("/append_entry", post(handle_append_entry))
            .route("/foobar", post(|| async { "foobar" }))
            .with_state(app_state),
    );
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed");
}

use axum::debug_handler;

#[debug_handler]
async fn handle_append_entry(
    State(app_state): State<AppState>,
    Json(message): Json<AppendEntry>,
    // ) -> Json<AppendEntryResponse> {
) -> impl IntoResponse {
    // app_state.handle_append(message);
    ""
}
