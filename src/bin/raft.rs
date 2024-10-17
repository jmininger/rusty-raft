use std::{
    env,
    net::SocketAddr,
    sync::Arc,
};

use color_eyre::Result;
use reqwest::{
    IntoUrl,
    Url,
};
use rusty_raft::{
    // json_rpc::{
    //     Request as RpcRequest,
    //     Response as RpcResponse,
    // },
    network::NetworkManager,
};
use serde::Serialize;
use tokio::{
    net::TcpListener,
    sync::Mutex,
};
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
struct Config {
    orchestrator_addr: SocketAddr,
    local_addr: SocketAddr,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {{ orchestrator_addr: {}, local_addr: {} }}",
            self.orchestrator_addr, self.local_addr
        )
    }
}

fn config_from_env() -> Result<Config> {
    dotenv::dotenv()?;
    let orchestrator_addr: SocketAddr = env::var("ORCHESTRATOR_ADDR")?.parse()?;
    let local_addr: SocketAddr = env::var("LOCAL_ADDR")?.parse()?;
    Ok(Config {
        orchestrator_addr,
        local_addr,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let conf @ Config {
        orchestrator_addr,
        local_addr,
    } = config_from_env()?;

    tracing::info!("Starting node with config: {}", conf);

    let orchestrator_url = Url::parse(&format!("http://{}/", orchestrator_addr))?;

    let network: Arc<Mutex<NetworkManager>> = Default::default();

    // Start listener for new tcp connections
    let server_handle = tokio::spawn({
        let network = network.clone();
        async move {
            let listener = TcpListener::bind(local_addr)
                .await
                .expect("Failed to bind to local address");
            while let Ok((raw_sock, addr)) = listener.accept().await {
                let mut network = network.lock().await;
                network.handle_new_inbound_connection(addr, raw_sock);
            }
        }
    });

    //Dial peers we received from orchestrator
    let peers_to_dial = get_peers(local_addr, orchestrator_url).await?;
    dial_peers(peers_to_dial, network.clone()).await?;

    // TODO: handle incoming requests
    // Some(req) = conn_mgr.incoming_requests() => {

    server_handle.await?;
    Ok(())
}

async fn dial_peers(peer_list: Vec<SocketAddr>, network: Arc<Mutex<NetworkManager>>) -> Result<()> {
    let mut network = network.lock().await;
    for peer in peer_list {
        if let Err(e) = network.dial_peer(peer).await {
            tracing::warn!("Failed to dial peer {}: {}", peer, e);
        }
    }
    tracing::debug!("Finished dialing initial peers");
    Ok(())
}

async fn get_peers(
    my_addr: impl Serialize,
    orchestrator_addr: impl IntoUrl,
) -> Result<Vec<SocketAddr>> {
    let addresses: Vec<SocketAddr> = reqwest::Client::new()
        .post(orchestrator_addr)
        .json(&serde_json::json!( { "address": my_addr } ))
        .send()
        .await?
        .json()
        .await?;
    Ok(addresses)
}
