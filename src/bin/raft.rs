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

#[derive(Debug)]
struct Config {
    orchestrator_addr: SocketAddr,
    local_addr: SocketAddr,
    node_id: u8,
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
    let node_id: u8 = env::var("NODE_ID")?.parse()?;
    Ok(Config {
        orchestrator_addr,
        local_addr,
        node_id,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let conf @ Config {
        orchestrator_addr,
        local_addr,
        node_id,
    } = config_from_env()?;

    tracing::info!("Starting node with config: {}", conf);

    let orchestrator_url = Url::parse(&format!("http://{}/", orchestrator_addr))?;

    let host_id = PeerId {
        dial_addr: local_addr,
        common_name: format!("node-{}", node_id),
    };
    let network: Arc<Mutex<NetworkManager>> =
        Arc::new(Mutex::new(NetworkManager::new(host_id.clone())));

    let host_id_clone = host_id.clone();
    let network_clone = network.clone();

    // Start listener for new tcp connections
    let server_handle = tokio::spawn({
        let host_id = host_id_clone;
        let network = network_clone;
        async move {
            let listener = TcpListener::bind(host_id.dial_addr)
                .await
                .expect("Failed to bind to local address");
            while let Ok((raw_sock, _addr)) = listener.accept().await {
                let hid = host_id.clone();
                let net = network.clone();
                tokio::spawn(async move {
                    rusty_raft::network::handle_new_inbound_connection(net, hid, raw_sock).await;
                });
            }
        }
    });

    //Dial peers we received from orchestrator
    let peers_to_dial = get_peers(host_id.clone(), orchestrator_url).await?;
    dial_peers(peers_to_dial, host_id, network.clone()).await?;

    // TODO: handle incoming requests
    // Some(req) = conn_mgr.incoming_requests() => {

    server_handle.await?;
    Ok(())
}

async fn dial_peers(
    peer_list: Vec<PeerId>,
    host_id: PeerId,
    network: Arc<Mutex<NetworkManager>>,
) -> Result<()> {
    for peer in peer_list {
        if let Err(e) = dial_peer(network.clone(), host_id.clone(), peer.dial_addr.clone()).await {
            tracing::warn!("Failed to dial addr {}: {}", peer, e);
        }
    }
    tracing::debug!("Finished dialing initial peers");
    Ok(())
}

async fn get_peers(host_id: PeerId, orchestrator_addr: impl IntoUrl) -> Result<Vec<PeerId>> {
    let addresses: Vec<PeerId> = reqwest::Client::new()
        .post(orchestrator_addr)
        .json(&host_id)
        .send()
        .await?
        .json()
        .await?;
    Ok(addresses)
}
