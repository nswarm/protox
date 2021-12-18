use anyhow::{Context, Result};

pub use client::supported_languages as client_supported_languages;
pub use direct::supported_languages as direct_supported_languages;
pub use proto::supported_languages as proto_supported_languages;
pub use server::supported_languages as server_supported_languages;

use crate::Config;

mod input;
mod proto;
mod direct;
mod client;
mod server;
mod util;
mod protoc;

pub fn configured(config: &Config) -> Result<()> {
    let input_files = input::collect(config).context("Failed to collect input files.")?;
    proto::run(config, &input_files)?;
    direct::run(config, &input_files)?;
    client::run(config, &input_files)?;
    server::run(config, &input_files)?;
    Ok(())
}
