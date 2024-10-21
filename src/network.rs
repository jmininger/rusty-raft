//! NetworkManager is responsible for managing all connections to peers
//! and handling incoming requests from peers.

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use color_eyre::Result;
use futures::SinkExt;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc,
        oneshot,
        Mutex,
    },
    task::JoinSet,
};
use tokio_stream::{
    wrappers::ReceiverStream,
    StreamExt,
    StreamMap,
};
use tokio_util::codec::{
    Framed,
    LinesCodec,
};

use crate::{
    connection::{
        ConnectionActor,
        ConnectionHandle,
    },
    peer::PeerId,
};
use crate::{
    json_rpc::{
        RpcRequest,
        // RequestId,
        RpcResponse,
    },
    peer::PeerName,
};

// async fn run_it_all() {
//     let mut conn_mgr = NetworkManager::new();
//     let local_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
//     let listener = TcpListener::bind(local_addr).await.unwrap();
//     loop {
//         tokio::select! {

//             Ok((raw_sock, addr)) = listener.accept() => {
//                 conn_mgr.handle_new_connection(addr, raw_sock);
//                 todo!();
//             },
//             Some(req) = conn_mgr.incoming_requests() => {
//                 todo!();
//             },
//         }
//     }
// }

/// Stream of inbound RpcRequests from a peer, with a oneshot channel to send the response
type InboundReqListener = ReceiverStream<(RpcRequest, oneshot::Sender<RpcResponse>)>;

pub struct NetworkManager {
    host_id: PeerId,
    /// Read handles, multiplexed together into a single event stream via the StreamMap
    connection_map: StreamMap<PeerName, InboundReqListener>,
    /// Write handles to connections
    connection_handles: HashMap<PeerName, ConnectionHandle>,

    peer_list: Vec<PeerId>,
}

impl NetworkManager {
    pub fn new(host_id: PeerId) -> Self {
        Self {
            host_id,
            connection_map: StreamMap::new(),
            connection_handles: HashMap::new(),
            peer_list: Vec::new(),
        }
    }

    pub fn list_connections(&self) -> Vec<PeerId> {
        //note: this assumes that connection_map and connection_handles should always have the same
        //keys and are in sync
        self.peer_list.clone()
    }

    pub async fn broadcast(
        &mut self,
        msg: RpcRequest,
        timeout_ms: Duration,
    ) -> Vec<(PeerName, Option<RpcResponse>)> {
        let mut responses = JoinSet::new();
        for (peer, conn) in self.connection_handles.iter() {
            let (send_resp, res_alert) = oneshot::channel();
            if let Err(send_err) = conn.send((msg.clone(), send_resp)).await {
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
        responses.join_all().await
    }

    /// Multiplex incoming requests from all connections into a single Stream
    /// Returns None if there are no more connections
    pub async fn incoming_requests(
        &mut self,
    ) -> Option<(PeerName, (RpcRequest, oneshot::Sender<RpcResponse>))> {
        self.connection_map.next().await
    }

    pub fn remove_connection(&mut self, peer: PeerName) {
        self.connection_map.remove(&peer);
        self.connection_handles.remove(&peer);
    }

    /// Takes a new socket connection and spins up a new ConnectionActor to manage it
    pub fn add_new_peer(&mut self, peer_id: &PeerId, raw_sock: TcpStream) {
        let peer = peer_id.common_name.clone();
        let (outbound_req_handle, outbound_req_alert) = mpsc::channel(32);
        let (inbound_req_handle, inbound_req_alert) = mpsc::channel(32);

        let actor = ConnectionActor::new(
            self.host_id.clone(),
            peer_id.common_name.clone(),
            raw_sock,
            outbound_req_alert,
            inbound_req_handle,
        );

        // Add both the read/write handles to the network
        let prev_conn = self
            .connection_map
            .insert(peer.clone(), ReceiverStream::new(inbound_req_alert));

        if prev_conn.is_some() {
            tracing::warn!(
                "Connection already exists for peer {}; Replacing it with new one",
                peer
            );
        }

        // Insert into `connection_handles` regardless of whether the key was present
        self.connection_handles.insert(peer, outbound_req_handle);
        self.peer_list.push(peer_id.clone());

        tracing::info!("Added new connection from peer {} to network", peer_id);

        // Manage the actor in the background
        tokio::spawn(async move { actor.run().await });
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
            (Some(Ok(name)), Some(Ok(addr))) => Ok(PeerId {
                common_name: name,
                dial_addr: addr.parse().unwrap(),
            }),
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
    network: Arc<Mutex<NetworkManager>>,
    host_id: PeerId,
    peer_addr: SocketAddr,
) -> Result<()> {
    tracing::info!("Dialing address {}", peer_addr);
    let raw_sock = TcpStream::connect(peer_addr).await?;
    let (raw_sock, peer_id) = exchange_identities(host_id, raw_sock).await?;

    tracing::debug!("Exchanged identity to peer {}", peer_id,);

    network.lock().await.add_new_peer(&peer_id, raw_sock);
    tracing::info!("Sucessfully dialed peer {}", peer_id);
    Ok(())
}

pub async fn handle_new_inbound_connection(
    network: Arc<Mutex<NetworkManager>>,
    host_id: PeerId,
    raw_sock: TcpStream,
) {
    let (raw_sock, peer_id) = exchange_identities(host_id, raw_sock).await.unwrap();
    tracing::info!("Received new inbound connection from {}", peer_id);
    network.lock().await.add_new_peer(&peer_id, raw_sock);
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
            return client_id;
        });
        let dialer_handle = tokio::spawn(async move {
            let host_id = PeerId {
                dial_addr: client_addr,
                common_name: "client".to_string(),
            };
            let raw_sock = TcpStream::connect(server_addr).await.unwrap();
            let (_raw_sock, server_id) = exchange_identities(host_id, raw_sock).await.unwrap();
            return server_id;
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
