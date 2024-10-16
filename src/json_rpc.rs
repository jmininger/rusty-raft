//! Types for handling json rpc

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

impl From<u64> for RequestId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: RequestId,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: RequestId,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
}
