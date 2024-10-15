//! Connection

use std::{
    collections::HashMap,
    net::SocketAddr,
};

use futures::SinkExt;
use tokio::{
    net::{
        tcp::{
            OwnedReadHalf,
            OwnedWriteHalf,
        },
        TcpStream,
    },
    sync::{
        mpsc,
        oneshot,
    },
};
use tokio_serde::{
    formats::SymmetricalJson,
    SymmetricallyFramed,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{
    FramedRead,
    FramedWrite,
    LengthDelimitedCodec,
};

use crate::json_rpc::{
    Request as RpcRequest,
    RequestId,
    Response as RpcResponse,
};

// type PeerId = SocketAddr;

pub type JsonWriteFrame = SymmetricallyFramed<
    FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    serde_json::Value,
    SymmetricalJson<serde_json::Value>,
>;

pub type JsonReadFrame = SymmetricallyFramed<
    FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    serde_json::Value,
    SymmetricalJson<serde_json::Value>,
>;

/// Allows us to send requests and await responses via oneshot
pub type ConnectionHandle = mpsc::Sender<(RpcRequest, oneshot::Sender<RpcResponse>)>;

/// Represents a single live connection to a peer

//////////////////////////////////////////////
// TODO: I currently have no way to send requests up the chain from single connection to the
// connection manager
//////////////////////////////////////////////
pub struct ConnectionActor {
    /// Socket address of the peer
    addr: SocketAddr,
    /// Resource write half of socket
    write_conn: JsonWriteFrame,
    ///reader
    read_conn: JsonReadFrame,
    /// Maps requests and
    requests: HashMap<RequestId, oneshot::Sender<serde_json::Value>>,
    /// req_alert listens for incoming requests
    // NOTE: I'm not yet sure if this just listens for new requests to fill or if it also listens
    // for responses to send out
    req_alert: mpsc::Receiver<(RpcRequest, oneshot::Sender<RpcResponse>)>,

    request_id: u64,
    // request_id: RequestId,
}

impl ConnectionActor {
    pub fn new(
        addr: SocketAddr,
        raw_sock: TcpStream,
        req_alert: mpsc::Receiver<(RpcRequest, oneshot::Sender<RpcResponse>)>,
    ) -> Self {
        let (read_raw, write_raw) = raw_sock.into_split();

        let write_conn: JsonWriteFrame = SymmetricallyFramed::new(
            FramedWrite::new(write_raw, LengthDelimitedCodec::new()),
            SymmetricalJson::<serde_json::Value>::default(),
        );

        let read_conn = SymmetricallyFramed::new(
            FramedRead::new(read_raw, LengthDelimitedCodec::new()),
            SymmetricalJson::<serde_json::Value>::default(),
        );
        Self {
            addr,
            write_conn,
            read_conn,
            req_alert,
            requests: HashMap::new(),
            request_id: 0,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some((req, resp_trigger)) = self.req_alert.recv() => {
                    todo!();
                    // self.handle_outgoing_req(msg).await;
                },
                Some(msg) = self.read_conn.next() => {
                    match msg {
                        Ok(msg) => {
                            todo!();
                            // self.handle_response(msg).await;
                        },
                        Err(e) => {
                            tracing::error!("Error reading from peer {}: {}", self.addr, e);
                        }
                    }
                },
            }
        }
    }

    async fn send_request(
        &mut self,
        msg: serde_json::Value,
        tx: oneshot::Sender<serde_json::Value>,
    ) {
        let id = self.request_id();
        if let Some(_) = self.requests.insert(id.clone(), tx) {
            tracing::error!("Duplicate request id {} for peer {}", id.0, self.addr);
        }
        self.write_conn
            .send(msg)
            .await
            .map_err(|_| {
                tracing::error!("Error sending request to peer {}", self.addr);
            })
            .unwrap();
    }

    // match self.requests.remove(&id) {
    //     Some(tx) => {
    //         tx.send(res).unwrap();
    //     },
    //     None => {
    //         tracing::error!("No request found for id {} for peer {}", id.0, self.addr);
    //     }
    // }

    fn request_id(&mut self) -> RequestId {
        let id = self.request_id;
        self.request_id += 1;
        RequestId(id)
    }
}
