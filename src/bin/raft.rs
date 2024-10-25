use std::time::Duration;

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
    json_rpc::RpcResponse,
    network::{
        self,
        dial_peer,
        NetworkManager,
    },
    peer::PeerId,
};
use tokio::{
    net::{
        TcpListener,
        TcpStream,
    },
    sync::mpsc,
};
use tracing_subscriber::EnvFilter;

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

    let host_id = PeerId {
        dial_addr: local_addr,
        common_name: format!("node-{}", node_id),
    };

    let orchestrator_url = orchestrator_addr.map(|addr| {
        Url::parse(&format!("http://{}/", addr)).expect("Failed to parse orchestrator url")
    });

    tracing::info!(
        "Starting node {} with config: {}",
        host_id.common_name,
        conf
    );

    // Setup application state
    let (req_trigger, mut req_alert) = mpsc::channel(100);
    let mut network = NetworkManager::new(host_id.clone(), req_trigger);

    // Run the server and forward new connections to the event loop for processing
    let (new_conn_trigger, mut new_conn_alert) = mpsc::channel(100);
    tokio::spawn(run_server(host_id.clone(), new_conn_trigger));

    // If an orchestrator_addr was passed into the config, contact it and get a list of peers to
    // dial
    if let Some(orchestrator_url) = orchestrator_url {
        if let Err(e) = bootstrap_peers_timeout(
            host_id.clone(),
            orchestrator_url,
            &mut network,
            Duration::from_secs(2),
        )
        .await
        {
            tracing::warn!("Failed to bootstrap peers: {}", e);
        }
    }

    loop {
        tracing::info!("Waiting for new connection or incoming request");
        tokio::select! {
            Some(raw_sock) = new_conn_alert.recv() => {
                let hid = host_id.clone();
                network::handle_new_inbound_connection(&mut network, hid, raw_sock).await;
            },
            Some((peer, (req, resp_trigger))) = req_alert.recv() => {
                tracing::info!("Received request {:?} from peer: {}", req, peer);
                let res = resp_trigger.send(RpcResponse { id: req.id, result: serde_json::json!{"received your request" }});
                if let Err(e) = res {
                    tracing::warn!("Failed to send response to peer: {:?}", e);
                }
            },
        }
    }

    // server_handle.await?;
    // Ok(())
}

async fn run_server(host_id: PeerId, new_conn_trigger: mpsc::Sender<TcpStream>) {
    let listener = TcpListener::bind(host_id.dial_addr)
        .await
        .expect("Failed to bind to local address");
    while let Ok((raw_sock, _addr)) = listener.accept().await {
        new_conn_trigger
            .send(raw_sock)
            .await
            .expect("Failed to send new connection");
    }
}

async fn dial_peers(
    peer_list: Vec<PeerId>,
    host_id: PeerId,
    network: &mut NetworkManager,
    timeout: Duration,
) -> Result<()> {
    for peer in peer_list {
        let dial_fut = dial_peer(network, host_id.clone(), peer.dial_addr);
        tokio::select! {
            res = dial_fut => {
                if let Err(e) = res {
                    tracing::warn!("Failed to dial addr {}: {}", peer, e);
                }
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

async fn bootstrap_peers_timeout(
    host_id: PeerId,
    orchestrator_addr: impl IntoUrl,
    network: &mut NetworkManager,
    dial_timeout: Duration,
) -> Result<()> {
    let peers_to_dial = bootstrap_peer_list(host_id.clone(), orchestrator_addr).await?;
    dial_peers(peers_to_dial, host_id.clone(), network, dial_timeout).await?;
    Ok(())
}
