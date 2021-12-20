use crate::Config;
use anyhow::{Context, Result};
use prost::Message;
use prost_types::FileDescriptorSet;
use std::fs;
use std::path::Path;

pub mod client;
pub mod direct;
pub mod proto;
pub mod server;

mod config;
mod renderer;

pub fn generate(config: &Config) -> Result<()> {
    let _descriptor_set = read_descriptor_set(&config.descriptor_set_path)?;
    // let _context = context::generate(&descriptor_set);
    proto::generate(config)?;
    direct::generate(config)?;
    client::generate(config)?;
    server::generate(config)?;
    Ok(())
}

fn read_descriptor_set(path: &Path) -> Result<FileDescriptorSet> {
    let bytes = fs::read(path).context("Failed to read file descriptor set.")?;
    let descriptor_set = Message::decode(bytes.as_slice())?;
    Ok(descriptor_set)
}
