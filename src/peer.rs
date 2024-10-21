use std::{
    fmt,
    net::SocketAddr,
};

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PeerId {
    pub common_name: PeerName,
    pub dial_addr: SocketAddr,
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.common_name, self.dial_addr)
    }
}

pub type PeerName = String;
