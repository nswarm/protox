mod context;
mod primitive;
mod renderer;
mod template_config;

pub mod client;
pub mod direct;
pub mod proto;
pub mod server;

pub use template_config::TemplateConfig;

use crate::Config;
use anyhow::Result;

pub fn generate(app_config: &Config) -> Result<()> {
    proto::generate(app_config)?;
    direct::generate(app_config)?;
    // client::generate(app_config)?;
    // server::generate(app_config)?;
    Ok(())
}
