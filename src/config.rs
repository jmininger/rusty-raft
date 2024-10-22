use std::{
    env,
    net::SocketAddr,
};

use color_eyre::Result;

#[derive(Debug)]
pub struct Config {
    pub orchestrator_addr: Option<SocketAddr>,
    pub local_addr: SocketAddr,
    pub node_id: u8,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(orchestrator_addr) = self.orchestrator_addr {
            write!(
                f,
                "Config {{ orchestrator_addr: {}, local_addr: {}, node_id: {} }}",
                orchestrator_addr, self.local_addr, self.node_id
            )
        } else {
            write!(
                f,
                "Config {{ orchestrator_addr: None, local_addr: {}, node_id: {} }}",
                self.local_addr, self.node_id
            )
        }
    }
}

pub fn config_from_env() -> Result<Config> {
    dotenv::dotenv()?;
    let orchestrator_addr: Option<SocketAddr> = env::var("ORCHESTRATOR_ADDR")
        .ok()
        .and_then(|addr| addr.parse().ok());
    let local_addr: SocketAddr = env::var("LOCAL_ADDR")?.parse()?;
    let node_id: u8 = env::var("NODE_ID")?.parse()?;
    Ok(Config {
        orchestrator_addr,
        local_addr,
        node_id,
    })
}
