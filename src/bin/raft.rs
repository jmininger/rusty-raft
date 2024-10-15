use color_eyre::Result;
use rusty_raft::json_rpc::{
    Message,
    Request as RpcRequest,
    Response as RpcResponse,
};

#[tokio::main]
async fn main() -> Result<()> {
    let req = Message::Request(RpcRequest {
        id: 1.into(),
        method: "foo".to_string(),
        params: serde_json::json!({"bar": "baz"}),
    });

    let req = serde_json::to_string(&req)?;
    println!("req: {}", req);

    Ok(())
}
