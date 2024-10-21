//! Types for handling JSON-RPC messages and requests in the context of peer-to-peer communication.
//! This module defines structures and enums for representing JSON-RPC requests, responses, and
//! associated metadata such as peer identification and message IDs.

use serde::{
    Deserialize,
    Serialize,
};

use crate::peer::PeerName;

/// Represents a full message sent between peers, containing both the JSON-RPC data and peer-related
/// metadata.
///
/// Each message consists of the `rpc` field, which contains either a request or a response, and the
/// `peer_id`, which identifies the peer sending or receiving the message. The `peer_id` allows
/// tracking which peer is associated with each message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub rpc: RpcMessage,
    pub peer_name: PeerName,
}

/// A wrapper around `u64` to represent the unique identifier for JSON-RPC requests and responses.
///
/// This ID is used to match requests with their corresponding responses in a JSON-RPC communication
/// session. It is guaranteed to be unique for each request-response pair.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

/// Implements the conversion from a `u64` to a `RequestId`.
///
/// This allows creating a `RequestId` directly from a `u64` value, simplifying the conversion
/// process.
impl From<u64> for RequestId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Represents a JSON-RPC request.
///
/// Each request consists of a unique `id`, the `method` to be invoked, and the `params` to be
/// passed to that method. The `params` are represented as a generic `serde_json::Value`, allowing
/// flexibility in the types of parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: RequestId,
    pub method: String,
    pub params: serde_json::Value,
}

/// Represents a JSON-RPC response.
///
/// Each response contains the unique `id` that matches it to the original request, and the `result`
/// of executing the requested method. The result is represented as a `serde_json::Value`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: RequestId,
    pub result: serde_json::Value,
}

/// Enum representing a complete JSON-RPC message
///
/// This enum allows the transport of either an `RpcRequest` or an `RpcResponse` in a single
/// message, facilitating communication between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcMessage {
    /// A JSON-RPC request message.
    Request(RpcRequest),
    /// A JSON-RPC response message.
    Response(RpcResponse),
}
