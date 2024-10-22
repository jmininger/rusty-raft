use std::{
    env,
    net::SocketAddr,
};

use color_eyre::Result;

#[derive(Debug)]
pub struct Config {
    pub orchestrator_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub node_id: u8,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {{ orchestrator_addr: {}, local_addr: {} }}",
            self.orchestrator_addr, self.local_addr
        )
    }
}

pub fn config_from_env() -> Result<Config> {
    dotenv::dotenv()?;
    let orchestrator_addr: SocketAddr = env::var("ORCHESTRATOR_ADDR")?.parse()?;
    let local_addr: SocketAddr = env::var("LOCAL_ADDR")?.parse()?;
    let node_id: u8 = env::var("NODE_ID")?.parse()?;
    Ok(Config {
        orchestrator_addr,
        local_addr,
        node_id,
    })
}
