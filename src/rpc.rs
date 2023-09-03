use crate::peer::*;
use crate::utils::*;

use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Alias makes it easy to swap things out for now
type AppState = ();

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

// Note we need to have an rpc for normal clients to add messages to the leader;
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
    // async fn request_vote(&self, target: NodeId, term: u64) {}
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

pub async fn run_rpc_server(port: u16) {
    let app = Router::new()
        .nest(
            "/rpc",
            Router::new()
                .route("/append_entry", post(handle_append_entry))
                .route("/foobar", post(|| async { "foobar" })),
        )
        .with_state(());
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed");
}

async fn handle_append_entry(
    State(app_state): State<AppState>,
    Json(message): Json<AppendEntry>,
    // ) -> Json<AppendEntryResponse> {
) -> Json<String> {
    // app_state.handle_append(message);
    Json("".into())
}
