use crate::generator::renderer::Renderer;
use crate::generator::template_config::TemplateConfig;
use crate::Config;
use anyhow::{Context, Result};
use prost::Message;
use prost_types::FileDescriptorSet;
use std::path::Path;
use std::{fs, io};

pub mod client;
pub mod direct;
pub mod proto;
pub mod server;

mod context;
mod primitive;
mod renderer;
mod template_config;

pub fn generate(app_config: &Config) -> Result<()> {
    proto::generate(app_config)?;
    direct::generate(app_config)?;
    // client::generate(app_config)?;
    // server::generate(app_config)?;
    Ok(())
}
