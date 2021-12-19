use anyhow::{Context, Result};

pub use client::supported_languages as client_supported_languages;
pub use direct::supported_languages as direct_supported_languages;
pub use proto::supported_languages as proto_supported_languages;
pub use server::supported_languages as server_supported_languages;

use crate::run::protoc::Protoc;
use crate::Config;

mod client;
mod direct;
mod input;
mod proto;
mod protoc;
mod server;
mod util;

pub fn configured(config: &Config) -> Result<()> {
    let mut protoc = Protoc::new(config)?;
    protoc.add_input_files(&mut input::collect(config).context("Failed to collect input files.")?);
    proto::run(config, &mut protoc)?;
    direct::run(config, &mut protoc)?;
    client::run(config, &mut protoc)?;
    server::run(config, &mut protoc)?;
    protoc.execute()?;
    Ok(())
}
