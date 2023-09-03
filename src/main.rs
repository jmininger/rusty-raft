mod context;
mod log;
mod network;
mod peer;
mod raft_state;
mod rpc;
mod utils;

use clap::{Arg, Command};

use eyre::Error;
use std::collections::HashMap;

use peer::*;
use raft_state::RaftState;
use rpc::run_rpc_server;
// use tokio;

#[tokio::main]
async fn main() {
    let (port, is_leader) = parse_cli();
    let peer_map = read_peer_cfg().unwrap();
    let raft_state = RaftState::new(is_leader);
    let rpc_client = rpc::RpcClient::new(peer_map);

    run_rpc_server(port).await;
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

// fn read_peer_map() -> PeerMap {}

// struct Peer {
//     id: u64,
//     addr: String,
// }

// impl Peer {}

fn read_peer_cfg() -> Result<PeerMap, Error> {
    Ok(PeerMap::new(HashMap::new()))
}
