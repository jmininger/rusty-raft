use crate::utils::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Peer {
    id: NodeId,
    addr: String,
}

#[derive(Debug, Clone)]
pub struct PeerMap(HashMap<NodeId, Peer>);

impl PeerMap {
    pub fn new(hm: HashMap<NodeId, Peer>) -> Self {
        Self(hm)
    }
    pub fn fetch_address(&self, id: NodeId) -> &str {
        //TODO handle error
        self.0.get(&id).map(|peer| &peer.addr).unwrap()
    }
}
