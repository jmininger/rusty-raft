use clap::{Arg, Command};

use color_eyre::Result;
use rpc::run_rpc_server;
use std::collections::HashMap;
use std::sync::Arc;

use peer::*;
use raft_state::RaftState;
use rusty_raft::*;
// use tokio;

#[tokio::main]
async fn main() {
    let (port, is_leader) = parse_cli();
    let peer_map = read_peer_cfg().unwrap();
    let raft_state = RaftState::new(is_leader);
    let rpc_client = rpc::RpcClient::new(peer_map);
    let app_state = context::RaftProtocol {
        state: raft_state,
        rpc_client: Arc::new(rpc_client),
        // peers: PeerMap::new(HashMap::new()),
    };

    run_rpc_server(app_state, port).await;
    // pub struct RaftProtocol {
    //     db: Box<std::fs::File>,
    //     state: RaftState,
    //     rpc_client: Arc<RpcClient>,
    //     peers: PeerMap,
    // }
}

fn parse_cli() -> (u16, bool) {
    let matches = Command::new("rusty-raft")
        .version("0.0.0")
        // A flag that determines whether the node is set to the leader at the beginning
        .arg(
            Arg::new("leader")
                .long("leader")
                .action(clap::ArgAction::SetTrue)
                .help("Sets the node to be the leader at the beginning"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .value_parser(clap::value_parser!(u16))
                .help("Sets the PORT of the node"), // .value_parser(clap::value_parser!(u16)),
        )
        .get_matches();
    let is_leader = matches.get_flag("leader");
    let port = matches
        .get_one::<u16>("port")
        .expect("Port is not a valid u16");
    (port.clone(), is_leader)
}

fn read_peer_cfg() -> Result<PeerMap> {
    Ok(PeerMap::new(HashMap::new()))
}

// fn read_peer_map() -> PeerMap {}

// struct Peer {
//     id: u64,
//     addr: String,
// }

// impl Peer {}
