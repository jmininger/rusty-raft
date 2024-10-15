//! ConnectionManager is responsible for managing all connections to peers
//! and handling incoming requests from peers.

use std::net::SocketAddr;

use color_eyre::Result;
// use futures::{
//     Sink,
//     SinkExt,
//     Stream,
//     StreamExt,
// };
use tokio::net::{
    tcp::OwnedWriteHalf,
    TcpListener,
    TcpStream,
};
use tokio::sync::{
    mpsc,
    oneshot,
};
use tokio_serde::{
    formats::SymmetricalJson,
    SymmetricallyFramed,
};
use tokio_stream::{
    StreamExt,
    StreamMap,
};
use tokio_util::codec::{
    FramedRead,
    FramedWrite,
    LengthDelimitedCodec,
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
    let mut conn_mgr = ConnectionManager::new();
    let local_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(local_addr).await.unwrap();
    loop {
        tokio::select! {
            Ok((raw_sock, addr)) = listener.accept() => {
                conn_mgr.new_connection(addr, raw_sock);
            },
            Ok(msg) = conn_mgr.incoming_requests() => {
                // conn_mgr.handle_msg(msg).await;
            },
            //     conn_mgr.broadcast(msg).await;
            //
        }
    }
}

struct ConnectionManager {
    connection_map: StreamMap<SocketAddr, TcpStream>,
}

/// The bulk of the logic is contained in new_connection, which is responsible for running
/// indiviual connections to peers
impl ConnectionManager {
    fn new() -> Self {
        ConnectionManager {
            connection_map: StreamMap::new(),
        }
    }

    pub async fn incoming_requests(&self) {
        self.connection_map.next().await
    }

    // async fn dial_peer(addr: SocketAddr) -> _ {
    //     todo!()
    // }

    fn new_connection(&mut self, addr: SocketAddr, raw_sock: TcpStream) -> ConnectionHandle {
        let (handle, req_alert) = mpsc::channel(32);
        let actor = ConnectionActor::new(addr, raw_sock, req_alert);
        tokio::spawn(async move { actor.run().await });
        return handle;
    }

    /// Send a request to a peer and then wait for the response
    async fn send_request(&mut self, addr: SocketAddr, msg: RpcRequest) -> Result<RpcResponse> {
        let conn = self.lookup_or_dial(addr).await?;

        let (send_resp, res_alert) = oneshot::channel();
        if let Err(send_err) = conn.send((send_resp, msg)).await {
            tracing::error!("Error sending request to connection manager: {}", send_err);
            return Err(send_err.into());
        }
        res_alert.await.map_err(|rcv_err| {
            // TODO: Better msg -- just means that the channel got dropped or used already, not
            // necessarily that the connection dropped
            tracing::warn!(
                "Connection with peer {} dropped before response received",
                addr.clone()
            );
            rcv_err.into()
        })
    }

    async fn lookup_or_dial(&mut self, addr: SocketAddr) -> Result<ConnectionHandle> {
        todo!()
    }

    // async fn broadcast(&mut self, msg: serde_json::Value) {
    //     for conn in &mut self.connections {
    //         todo!()
    //     }
    // }
}
