use std::{
    sync::Arc,
    time::Duration,
};

use color_eyre::Result;
use reqwest::{
    IntoUrl,
    Url,
};
use rusty_raft::{
    config::{
        config_from_env,
        Config,
    },
    network,
    network::{
        dial_peer,
        NetworkManager,
    },
    peer::PeerId,
};
use tokio::{
    net::TcpListener,
    sync::Mutex,
};
use tracing_subscriber::EnvFilter;

async fn dial_peers(
    peer_list: Vec<PeerId>,
    host_id: PeerId,
    network: Arc<Mutex<NetworkManager>>,
    timeout: Duration,
) -> Result<()> {
    for peer in peer_list {
        let dial_fut = dial_peer(network.clone(), host_id.clone(), peer.dial_addr.clone());
        tokio::select! {
            Err(e) = dial_fut => {
                tracing::warn!("Failed to dial addr {}: {}", peer, e);
            },
            _ = tokio::time::sleep(timeout) => {
                tracing::warn!("Dial of addr {} timed-out", peer);
            }
        }
    }
    tracing::debug!("Finished dialing initial peers");
    Ok(())
}

async fn bootstrap_peer_list(
    host_id: PeerId,
    orchestrator_addr: impl IntoUrl,
) -> Result<Vec<PeerId>> {
    let addresses: Vec<PeerId> = reqwest::Client::new()
        .post(orchestrator_addr)
        .json(&host_id)
        .send()
        .await?
        .json()
        .await?;
    Ok(addresses)
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Load in config from ENV
    let conf @ Config {
        orchestrator_addr,
        local_addr,
        node_id,
    } = config_from_env()?;

    let orchestrator_url = Url::parse(&format!("http://{}/", orchestrator_addr))?;

    let host_id = PeerId {
        dial_addr: local_addr,
        common_name: format!("node-{}", node_id),
    };

    tracing::info!(
        "Starting node {} with config: {}",
        host_id.common_name,
        conf
    );

    // Setup application state
    let network: Arc<Mutex<NetworkManager>> =
        Arc::new(Mutex::new(NetworkManager::new(host_id.clone())));

    // Start listener for new tcp connections so that other nodes can dial the host
    let server_handle = tokio::spawn({
        let host_id = host_id.clone();
        let network = Arc::clone(&network);
        async move {
            let listener = TcpListener::bind(host_id.dial_addr)
                .await
                .expect("Failed to bind to local address");
            while let Ok((raw_sock, _addr)) = listener.accept().await {
                let hid = host_id.clone();
                let net = Arc::clone(&network);
                tokio::spawn(async move {
                    network::handle_new_inbound_connection(net, hid, raw_sock).await;
                });
            }
        }
    });

    //Dial peers we received from orchestrator
    let peers_to_dial = bootstrap_peer_list(host_id.clone(), orchestrator_url).await?;
    let dial_timeout = Duration::from_secs(2);
    dial_peers(
        peers_to_dial,
        host_id.clone(),
        Arc::clone(&network),
        dial_timeout,
    )
    .await?;

    // TODO: handle incoming requests
    // Some(req) = conn_mgr.incoming_requests() => {

    server_handle.await?;
    Ok(())
}
