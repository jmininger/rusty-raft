//! NetworkManager is responsible for managing all connections to peers
//! and handling incoming requests from peers.

use std::{
    collections::HashMap,
    net::SocketAddr,
    time::Duration,
};

use color_eyre::Result;
use tokio::{
    net::{
        TcpListener,
        TcpStream,
    },
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

use crate::connection::{
    ConnectionActor,
    ConnectionHandle,
};
use crate::json_rpc::{
    Request as RpcRequest,
    // RequestId,
    Response as RpcResponse,
};

async fn run_it_all() {
    let mut conn_mgr = NetworkManager::new();
    let local_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(local_addr).await.unwrap();
    loop {
        tokio::select! {

            Ok((raw_sock, addr)) = listener.accept() => {
                conn_mgr.handle_new_connection(addr, raw_sock);
                todo!();
            },
            Some(req) = conn_mgr.incoming_requests() => {
                todo!();
            },
        }
    }
}

/// Stream of inbound RpcRequests from a peer, with a oneshot channel to send the response
type InboundReqListener = ReceiverStream<(RpcRequest, oneshot::Sender<RpcResponse>)>;

struct NetworkManager {
    /// Read handles, multiplexed together into a single event stream via the StreamMap
    connection_map: StreamMap<SocketAddr, InboundReqListener>,
    /// Write handles to connections
    connection_handles: HashMap<SocketAddr, ConnectionHandle>,
}

impl NetworkManager {
    fn new() -> Self {
        Self {
            connection_map: StreamMap::new(),
            connection_handles: HashMap::new(),
        }
    }

    pub fn list_connections(&self) -> Vec<SocketAddr> {
        //note: this assumes that connection_map and connection_handles should always have the same
        //keys
        self.connection_handles.keys().cloned().collect()
    }

    pub async fn broadcast(
        &mut self,
        msg: RpcRequest,
        timeout_ms: Duration,
    ) -> Vec<(SocketAddr, Option<RpcResponse>)> {
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
    ) -> Option<(SocketAddr, (RpcRequest, oneshot::Sender<RpcResponse>))> {
        self.connection_map.next().await
    }

    pub fn remove_connection(&mut self, addr: SocketAddr) {
        self.connection_map.remove(&addr);
        self.connection_handles.remove(&addr);
    }

    /// Open up a connection with a new peer
    pub async fn dial_peer(&mut self, addr: SocketAddr) -> Result<()> {
        let raw_sock = TcpStream::connect(addr).await?;
        self.handle_new_connection(addr, raw_sock);
        Ok(())
    }

    /// Takes a new socket connection and spins up a new ConnectionActor to manage it
    pub fn handle_new_connection(&mut self, addr: SocketAddr, raw_sock: TcpStream) {
        let (outbound_req_handle, outbound_req_alert) = mpsc::channel(32);
        let (inbound_req_handle, inbound_req_alert) = mpsc::channel(32);
        let actor = ConnectionActor::new(addr, raw_sock, outbound_req_alert, inbound_req_handle);
        if let Some(_) = self
            .connection_map
            .insert(addr.clone(), ReceiverStream::new(inbound_req_alert))
            .and_then(|_| {
                self.connection_handles
                    .insert(addr.clone(), outbound_req_handle)
            })
        {
            tracing::warn!(
                "Connection already exists for peer {}; Replacing it with new one",
                addr
            );
        }
        tokio::spawn(async move { actor.run().await });
    }
}
