//! NetworkManager is responsible for managing all connections to peers
//! and handling incoming requests from peers.

use std::{
    collections::HashMap,
    time::Duration,
};

use color_eyre::Result;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc,
        oneshot,
    },
    task::JoinSet,
};
use tokio_stream::{
    wrappers::ReceiverStream,
    StreamExt,
    StreamMap,
};

use crate::json_rpc::{
    Request as RpcRequest,
    // RequestId,
    Response as RpcResponse,
};
use crate::{
    connection::{
        ConnectionActor,
        ConnectionHandle,
    },
    peer::PeerId,
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

#[derive(Default)]
pub struct NetworkManager {
    /// Read handles, multiplexed together into a single event stream via the StreamMap
    connection_map: StreamMap<PeerId, InboundReqListener>,
    /// Write handles to connections
    connection_handles: HashMap<PeerId, ConnectionHandle>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            connection_map: StreamMap::new(),
            connection_handles: HashMap::new(),
        }
    }

    pub fn list_connections(&self) -> Vec<PeerId> {
        //note: this assumes that connection_map and connection_handles should always have the same
        //keys
        self.connection_handles.keys().cloned().collect()
    }

    pub async fn broadcast(
        &mut self,
        msg: RpcRequest,
        timeout_ms: Duration,
    ) -> Vec<(PeerId, Option<RpcResponse>)> {
        let mut responses = JoinSet::new();
        for (addr, conn) in self.connection_handles.iter() {
            let addr = addr.clone();
            let (send_resp, res_alert) = oneshot::channel();
            if let Err(send_err) = conn.send((msg.clone(), send_resp)).await {
                tracing::error!("Error sending request to connection manager: {}", send_err);
            }
            responses.spawn(async move {
                (
                    addr,
                    tokio::select! {
                        res = res_alert => res.map_err(|e| {
                            tracing::error!("Error with peer: {}: {}", addr, e);
                        }).ok(),
                        _ = tokio::time::sleep(timeout_ms) => {
                            tracing::warn!("Timed out waiting for response from peer {}", addr);
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
    ) -> Option<(PeerId, (RpcRequest, oneshot::Sender<RpcResponse>))> {
        self.connection_map.next().await
    }

    pub fn remove_connection(&mut self, addr: PeerId) {
        self.connection_map.remove(&addr);
        self.connection_handles.remove(&addr);
    }

    /// Open up a connection with a new peer
    pub async fn dial_peer(&mut self, addr: PeerId) -> Result<()> {
        tracing::debug!("Dialing peer {}", addr);
        let raw_sock = TcpStream::connect(addr).await?;
        self.handle_new_connection(addr, raw_sock);
        tracing::info!("Sucessfully dialed peer {}", addr);
        Ok(())
    }

    pub fn handle_new_inbound_connection(&mut self, addr: PeerId, raw_sock: TcpStream) {
        tracing::info!("Received new inbound connection from {}", addr);
        self.handle_new_connection(addr, raw_sock);
        tracing::debug!("Added new connection to peer {}", addr);
    }

    /// Takes a new socket connection and spins up a new ConnectionActor to manage it
    fn handle_new_connection(&mut self, addr: PeerId, raw_sock: TcpStream) {
        let (outbound_req_handle, outbound_req_alert) = mpsc::channel(32);
        let (inbound_req_handle, inbound_req_alert) = mpsc::channel(32);

        let actor = ConnectionActor::new(addr, raw_sock, outbound_req_alert, inbound_req_handle);

        // Add both the read/write handles to the network
        let prev_conn = self
            .connection_map
            .insert(addr.clone(), ReceiverStream::new(inbound_req_alert));

        if prev_conn.is_some() {
            tracing::warn!(
                "Connection already exists for peer {}; Replacing it with new one",
                addr
            );
        }

        // Insert into `connection_handles` regardless of whether the key was present
        self.connection_handles
            .insert(addr.clone(), outbound_req_handle);

        // Manage the actor in the background
        tokio::spawn(async move { actor.run().await });
    }
}
