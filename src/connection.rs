//!   ### Some important invariants:
//!     - A connection will not receive more than one request without receiving a response back
//!     - A connection CANNOT service both an inbound request and an outbound request at the same
//!       time -- TODO: Should double check whether this is a property we want given raft's flow --
//!       Nonetheless, it should simplify things for now

use std::error::Error;

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
        oneshot::{
            self,
        },
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

use crate::{
    json_rpc::{
        Message as RpcMessage,
        Request as RpcRequest,
        RequestId,
        Response as RpcResponse,
    },
    peer::PeerId,
    utils::dynamic_fut,
};

type JsonWriteFrame = SymmetricallyFramed<
    FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    RpcMessage,
    SymmetricalJson<RpcMessage>,
>;

type JsonReadFrame = SymmetricallyFramed<
    FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    RpcMessage,
    SymmetricalJson<RpcMessage>,
>;

/// [`ConnectionHandle`] allows us to send requests from the ConnectionActor to the rest of the
/// application
pub type ConnectionHandle = mpsc::Sender<(RpcRequest, ResponseHandle)>;

/// [`ResponseHandle`] allows us to send responses back to the peer/app
pub type ResponseHandle = oneshot::Sender<RpcResponse>;

/// [`ConnectionActor`] is responsible for handling the connection to a single peer. It parses
/// inbound requests/responses into [`JsonRpcMessage`] types and sends outbound requests/responses
/// to the peer
pub struct ConnectionActor {
    /// Socket address of the peer
    addr: PeerId,
    /// Write half of connection; Writes JsonRpcMessage types
    write_conn: JsonWriteFrame,
    /// Read half of connection; Read JsonRpcMessage types
    read_conn: JsonReadFrame,

    /// Used to send inbound requests to the application
    inbound_req_handle: ConnectionHandle,

    /// Used to listen for responses to requests that the application is servicing
    outbound_resp_alert: Option<oneshot::Receiver<RpcResponse>>,

    /// Used for the application to send requests to the peer over this connection
    outbound_req_alert: mpsc::Receiver<(RpcRequest, ResponseHandle)>,

    /// Used for notifying the application that the request that was sent out to the peer has been
    /// responded to
    active_outbound_request: Option<(RequestId, ResponseHandle)>,

    ///TODO: Not currently used anywhere
    _request_id: u64,
    // request_id: RequestId,
}

impl ConnectionActor {
    /// Creates Read/Write frames for the raw socket and initializes the ConnectionActor
    pub fn new(
        addr: PeerId,
        raw_sock: TcpStream,
        outbound_req_alert: mpsc::Receiver<(RpcRequest, ResponseHandle)>,
        inbound_req_handle: ConnectionHandle,
    ) -> Self {
        let (read_raw, write_raw) = raw_sock.into_split();

        let write_conn: JsonWriteFrame = SymmetricallyFramed::new(
            FramedWrite::new(write_raw, LengthDelimitedCodec::new()),
            SymmetricalJson::<RpcMessage>::default(),
        );

        let read_conn = SymmetricallyFramed::new(
            FramedRead::new(read_raw, LengthDelimitedCodec::new()),
            SymmetricalJson::<RpcMessage>::default(),
        );
        Self {
            addr,
            write_conn,
            read_conn,
            inbound_req_handle,
            outbound_req_alert,
            active_outbound_request: None,
            outbound_resp_alert: None,
            _request_id: 0,
        }
    }

    /// This is the only exposed function (excluding the constructor). The application calls this
    /// method every time it receives a new connection over the socket. Inbound and outbound
    /// responses are handled with oneshots. Multi-producer, single-consumer channels are used to
    /// communicate requests between the application and the [`ConnectionActor`]
    pub async fn run(mut self) {
        loop {
            let outbound_resp_alert = dynamic_fut(self.outbound_resp_alert.take());
            tokio::select! {
                res = outbound_resp_alert => self.handle_outbound_response(res).await,
                res = self.outbound_req_alert.recv() => self.handle_outbound_request(res).await,
                Some(msg) = self.read_conn.next() => self.handle_inbound_read(msg).await,
            }
        }
    }

    async fn handle_outbound_response(&mut self, res: Result<RpcResponse, impl Error>) {
        match res {
            Ok(resp) => {
                self.write_conn
                    .send(RpcMessage::Response(resp))
                    .await
                    .unwrap_or_else(|_| {
                        tracing::error!("Error sending response to peer {}", self.addr);
                    });
            }
            Err(_recv_err) => {
                todo!("Handle dropped");
            }
        }
    }

    async fn handle_outbound_request(&mut self, res: Option<(RpcRequest, ResponseHandle)>) {
        match res {
            Some((req, resp_trigger)) => {
                let id = req.id.clone();
                let msg = RpcMessage::Request(req);

                if self.active_outbound_request.is_some() {
                    tracing::error!("Duplicate request id {} for peer {}", id.0, self.addr);
                } else {
                    self.active_outbound_request = Some((id, resp_trigger));
                    self.write_conn.send(msg).await.unwrap_or_else(|_| {
                        tracing::error!("Error sending request to peer {}", self.addr);
                    });
                }
            }
            None => todo!("Handle dropped connection"),
        }
    }

    async fn handle_inbound_read(&mut self, msg: Result<RpcMessage, impl Error>) {
        match msg {
            Ok(msg) => {
                self.handle_inbound_message(msg).await;
            }
            Err(e) => {
                tracing::error!("Error reading from peer {}: {}", self.addr, e);
                todo!(
                    "Unwind, deallocate everything, and probably send a msg up to the \
                     ConnectionManager to let them know to remove"
                );
            }
        }
    }

    async fn handle_inbound_message(&mut self, msg: RpcMessage) {
        match msg {
            RpcMessage::Request(req) => {
                let (tx, rx) = oneshot::channel();
                match self.outbound_resp_alert {
                    Some(_) => {
                        tracing::error!(
                            "Received a second inbound request before responding to the first"
                        );
                    }
                    None => {
                        self.outbound_resp_alert = Some(rx);
                        self.inbound_req_handle
                            .send((req, tx))
                            .await
                            .expect("Inbound request handler dropped before response sent");
                    }
                }
            }
            RpcMessage::Response(resp) => {
                if let Some((outbound_id, resp_trigger)) = self.active_outbound_request.take() {
                    if outbound_id == resp.id {
                        resp_trigger
                            .send(resp)
                            .expect("Response handler dropped before response sent");
                    } else {
                        self.active_outbound_request = Some((outbound_id, resp_trigger));
                        tracing::error!(
                            "Received response with mismatched id from peer {}: expected {}, got \
                             {}",
                            self.addr,
                            outbound_id.0,
                            resp.id.0,
                        );
                    }
                } else {
                    tracing::error!(
                        "Received unexpected response from peer {}: {}",
                        self.addr,
                        resp.id.0,
                    );
                }
            }
        }
    }

    // fn request_id(&mut self) -> RequestId {
    //     let id = self.request_id;
    //     self.request_id += 1;
    //     RequestId(id)
    // }
}
