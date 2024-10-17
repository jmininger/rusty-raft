use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::post,
    Json,
    Router,
};
use color_eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::Mutex;
use tracing::info;

#[derive(Serialize, Deserialize)]
struct PeerRequest {
    address: SocketAddr,
}

#[derive(Clone, Default)]
struct AppState(Arc<Mutex<HashSet<SocketAddr>>>);

#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    host: SocketAddr,
}

/// Each peer hits the orchestrator with a POST request to add itself to the list of peers.
/// It receives back

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let Args { host } = clap::Parser::parse();

    let app_state = Default::default();
    let app = Router::new()
        .route("/", post(handle_orchestrator))
        .with_state(app_state);

    tracing::info!("Listening on {}", host.clone());

    axum::Server::bind(&host)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await
        .unwrap();

    Ok(())
}

async fn handle_orchestrator(
    State(state): State<AppState>,
    Json(req): Json<PeerRequest>,
) -> impl IntoResponse {
    info!("Adding {} to orchestrator peer list", req.address);
    let mut peers = state.0.lock().await;
    let mut addresses = peers.iter().cloned().collect::<Vec<_>>();

    // Insert the peer into the list and remove it from the list of it's peers if it ahs already
    // been reprsented in it
    if !peers.insert(req.address) {
        addresses.retain(|&addr| addr != req.address);
    }
    Json(addresses)
}
