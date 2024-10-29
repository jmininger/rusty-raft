//! NetworkManager is responsible for managing all connections to peers
//! and handling incoming requests from peers.

use std::{
    collections::HashMap,
    net::SocketAddr,
    time::Duration,
};

use color_eyre::Result;
use futures::SinkExt;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc,
        oneshot,
    },
    task::{
        AbortHandle,
        JoinSet,
    },
};
use tokio_stream::StreamExt;
use tokio_util::codec::{
    Framed,
    LinesCodec,
};

use crate::{
    connection::{
        ConnectionActor,
        ConnectionHandle,
        RequestTrigger,
    },
    json_rpc::{
        RpcRequest,
        RpcResponse,
    },
    peer::{
        PeerId,
        PeerName,
    },
};

/// The [`NetworkManager`]'s view of a ConnectionActor
struct Connection {
    pub peer_id: PeerId,
    pub handle: ConnectionHandle,
    pub abort_trigger: AbortHandle,
}

pub struct NetworkManager {
    host_id: PeerId,
    /// Write handles to connections
    connection_handles: HashMap<PeerName, Connection>,
    /// Write handles for connections to send request upstream to the app so that the app can
    /// manage them
    request_trigger: RequestTrigger,
}

impl NetworkManager {
    pub fn new(host_id: PeerId, request_trigger: RequestTrigger) -> Self {
        Self {
            host_id,
            request_trigger,
            connection_handles: HashMap::new(),
        }
    }

    pub fn list_connections(&self) -> Vec<PeerId> {
        self.connection_handles
            .values()
            .map(|c| c.peer_id.clone())
            .collect()
    }

    pub async fn broadcast(
        &mut self,
        msg: RpcRequest,
        timeout_ms: Duration,
    ) -> JoinSet<(PeerName, Option<RpcResponse>)> {
        let mut responses = JoinSet::new();
        for (peer, conn) in self.connection_handles.iter() {
            let (response_trigger, res_alert) = oneshot::channel();
            if let Err(send_err) = conn.handle.send((msg.clone(), response_trigger)).await {
                tracing::error!("Error sending request to connection manager: {}", send_err);
            }
            let peer = peer.clone();
            responses.spawn(async move {
                (
                    peer.clone(),
                    tokio::select! {
                        res = res_alert => res.map_err(|e| {
                            tracing::error!("Error with peer: {}: {}", peer, e);
                        }).ok(),
                        _ = tokio::time::sleep(timeout_ms) => {
                            tracing::warn!("Timed out waiting for response from peer {}", peer);
                            None
                        }
                    },
                )
            });
        }
        responses
    }

    pub fn remove_connection(&mut self, peer: PeerName) {
        self.connection_handles.remove(&peer);
    }

    /// Takes a new socket connection and spins up a new ConnectionActor to manage it
    pub fn add_new_peer(&mut self, peer_id: &PeerId, raw_sock: TcpStream) {
        let (outbound_req_handle, outbound_req_alert) = mpsc::channel(32);
        let inbound_req_handle = self.request_trigger.clone();
        let host_id = self.host_id.clone();
        let peer_name = peer_id.common_name.clone();

        // Spawn a new connection actor to manage the connection
        let join_handle = tokio::spawn(async move {
            ConnectionActor::new(
                host_id,
                peer_name,
                raw_sock,
                outbound_req_alert,
                inbound_req_handle,
            )
            .run()
            .await
        });

        self.add_connection(
            peer_id.clone(),
            outbound_req_handle,
            join_handle.abort_handle(),
        );
    }

    fn add_connection(
        &mut self,
        peer_id: PeerId,
        handle: ConnectionHandle,
        abort_trigger: AbortHandle,
    ) {
        let peer_name = peer_id.common_name.clone();
        let prev_conn = self.connection_handles.insert(
            peer_name.clone(),
            Connection {
                peer_id: peer_id.clone(),
                handle,
                abort_trigger,
            },
        );

        if let Some(conn) = prev_conn {
            tracing::warn!(
                "Connection already exists for peer {}; Replacing it with new one",
                peer_name
            );
            conn.abort_trigger.abort();
        }
        tracing::info!("Added new connection from peer {} to network", peer_id);
    }
}

pub async fn exchange_identities(
    host_id: PeerId,
    tcp_sock: TcpStream,
) -> Result<(TcpStream, PeerId)> {
    let lc = LinesCodec::new();
    let mut framed = Framed::new(tcp_sock, lc);
    framed.send(host_id.common_name).await?;
    framed.send(host_id.dial_addr.to_string()).await?;
    let get_id_fut = async {
        let name_line = framed.next().await;
        let addr_line = framed.next().await;
        match (name_line, addr_line) {
            (Some(Ok(name)), Some(Ok(addr))) => {
                let addr = match addr.parse() {
                    Ok(dial_addr) => dial_addr,
                    Err(e) => {
                        tracing::debug!("Failed to parse dial address: {}", e);
                        panic!("Failed to parse dial address");
                    }
                };
                Ok(PeerId {
                    common_name: name,
                    dial_addr: addr,
                })
            }
            _ => Err("Failed to read peer id"),
        }
    };
    tokio::select! {
        Ok(peer_id) = get_id_fut => {
            let raw_sock = framed.into_inner();
            Ok((raw_sock, peer_id))
        },
        _ = tokio::time::sleep(Duration::from_secs(1)) => {
            todo!("Timeout waiting for peer id")
        }
    }
}

pub async fn dial_peer(
    network: &mut NetworkManager,
    host_id: PeerId,
    peer_addr: SocketAddr,
) -> Result<()> {
    tracing::info!("Dialing address {}", peer_addr);
    let raw_sock = TcpStream::connect(peer_addr).await?;
    let (raw_sock, peer_id) = exchange_identities(host_id, raw_sock).await?;

    tracing::debug!("Exchanged identity to peer {}", peer_id,);

    network.add_new_peer(&peer_id, raw_sock);
    tracing::info!("Sucessfully dialed peer {}", peer_id);
    Ok(())
}

pub async fn handle_new_inbound_connection(
    network: &mut NetworkManager,
    host_id: PeerId,
    raw_sock: TcpStream,
) {
    let (raw_sock, peer_id) = exchange_identities(host_id, raw_sock).await.unwrap();
    tracing::info!("Received new inbound connection from {}", peer_id);
    network.add_new_peer(&peer_id, raw_sock);
    tracing::debug!("Added new connection to peer {}", peer_id);
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;

    use super::*;
    use crate::peer::PeerId;

    #[tokio::test]
    async fn test_identity() {
        let server_addr = "127.0.0.1:9001".parse().unwrap();
        let client_addr = "127.0.0.1:9002".parse().unwrap();

        let server_handle = tokio::spawn(async move {
            let host_id = PeerId {
                dial_addr: server_addr,
                common_name: "server".to_string(),
            };
            let server = TcpListener::bind(server_addr).await.unwrap();
            let (raw_sock, _addr) = server.accept().await.unwrap();
            let (_raw_sock, client_id) = exchange_identities(host_id, raw_sock).await.unwrap();
            client_id
        });
        let dialer_handle = tokio::spawn(async move {
            let host_id = PeerId {
                dial_addr: client_addr,
                common_name: "client".to_string(),
            };
            let raw_sock = TcpStream::connect(server_addr).await.unwrap();
            let (_raw_sock, server_id) = exchange_identities(host_id, raw_sock).await.unwrap();
            server_id
        });

        let (server_res, dialer_res) = tokio::join!(server_handle, dialer_handle);
        match (server_res, dialer_res) {
            // server_res should contain the dialer peer id and the dialer_res should contain the
            // server id
            (Ok(dialer_id), Ok(server_id)) => {
                assert_eq!(server_id.common_name, "server");
                assert_eq!(dialer_id.common_name, "client");
            }
            _ => panic!("Failed to exchange identities"),
        }
    }
}
