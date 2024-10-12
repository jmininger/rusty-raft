use std::{
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::State,
    http::StatusCode,
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
use tokio::sync::{
    broadcast,
    mpsc,
};
use tracing::info;

#[derive(Serialize, Deserialize)]
struct PeerRequest {
    address: SocketAddr,
}

#[derive(Clone)]
struct AppState(
    mpsc::Sender<SocketAddr>,
    Arc<broadcast::Receiver<Vec<SocketAddr>>>,
);

#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    num_nodes: usize,

    #[arg(long)]
    host: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let Args { num_nodes, host } = clap::Parser::parse();

    let (new_addr_tx, mut new_addr_rx) = tokio::sync::mpsc::channel(num_nodes);
    let (alert, alert_rx) = tokio::sync::broadcast::channel(1);
    let app_state = AppState(new_addr_tx, Arc::new(alert_rx));

    //TODO: Add a layer timeout
    let app = Router::new()
        .route("/", post(handle_orchestrator))
        .with_state(app_state);

    // State managment task
    // This task will listen for new addresses and once it reaches the threshold
    // of total nodes it will send the addresses to the listening requesters
    tokio::spawn(async move {
        let alert = alert.clone();
        let mut state = Vec::new();
        while let Some(elem) = new_addr_rx.recv().await {
            state.push(elem);
            if state.len() == num_nodes {
                alert.send(state.clone()).unwrap();
                break;
            }
        }
        tracing::warn!("Exiting orchestator loop");
    });

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
    state.0.send(req.address).await.unwrap();

    // Hang until all nodes have been spun up and communicate with the orchestrator binary
    let res = state.1.resubscribe().recv().await;

    match res {
        Ok(addresses) => Ok(Json(addresses)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
