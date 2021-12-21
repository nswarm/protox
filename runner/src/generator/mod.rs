use crate::generator::renderer::Renderer;
use crate::generator::template_config::TemplateConfig;
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

mod context;
mod primitive;
mod renderer;
mod template_config;

pub fn generate(app_config: &Config) -> Result<()> {
    let descriptor_set = read_descriptor_set(&app_config.descriptor_set_path)?;
    let template_config = TemplateConfig::default(); // todo load from file.
    let renderer = Renderer::with_config(template_config);
    // todo render(descriptor_set);
    proto::generate(app_config)?;
    // direct::generate(app_config)?;
    // client::generate(app_config)?;
    // server::generate(app_config)?;
    Ok(())
}

fn read_descriptor_set(path: &Path) -> Result<FileDescriptorSet> {
    let bytes = fs::read(path).context("Failed to read file descriptor set.")?;
    let descriptor_set = Message::decode(bytes.as_slice())?;
    Ok(descriptor_set)
}
