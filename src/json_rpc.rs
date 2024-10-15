//! Types for handling json rpc

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    id: RequestId,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    id: RequestId,
    result: serde_json::Value,
}
