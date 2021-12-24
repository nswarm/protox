use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

mod config;
mod idl;
mod lang;
mod lang_config;
mod protoc;
mod template_config;
mod util;

pub mod template_renderer;

use crate::util::DisplayNormalized;
pub use config::Config;
pub use idl::Idl;
pub use lang::Lang;

pub fn run() -> Result<()> {
    env_logger::init();
    let config = Config::from_cli()?;
    run_internal(config)
}

pub fn run_with_config(config: Config) -> Result<()> {
    env_logger::init();
    run_internal(config)
}

fn run_internal(config: Config) -> Result<()> {
    create_output_root(config.output_root.as_ref())?;
    match config.idl {
        Idl::Proto => {
            protoc::generate(&config)?;
            template_renderer::generate(&config)?;
        }
    };

    Ok(())
}

fn create_output_root(path: Option<&PathBuf>) -> Result<()> {
    match path {
        None => Ok(()),
        Some(path) => Ok(fs::create_dir_all(path).with_context(|| {
            format!(
                "Failed to create output directories in path: {}",
                path.display_normalized()
            )
        })?),
    }
}
