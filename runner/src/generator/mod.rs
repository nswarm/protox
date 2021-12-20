use crate::Config;
use anyhow::Result;

pub mod client;
pub mod direct;
pub mod proto;
pub mod server;

pub fn generate(config: &Config) -> Result<()> {
    proto::generate(config)?;
    direct::generate(config)?;
    client::generate(config)?;
    server::generate(config)?;
    Ok(())
}
