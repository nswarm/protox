#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

mod config;
mod idl;
mod lang;
mod lang_config;
mod protoc;
mod template_config;
mod template_init;
mod template_renderer;
mod util;

use crate::template_init::initialize_template_dir;
use crate::template_renderer::CONFIG_FILE_NAME;
use crate::util::DisplayNormalized;
pub use config::Config;
pub use idl::Idl;
pub use lang::Lang;

pub fn run() -> Result<()> {
    env_logger::init();
    let config = Config::from_cli()?;
    run_internal(&config)
}

pub fn run_with_config(config: Config) -> Result<()> {
    env_logger::init();
    run_internal(&config)
}

fn run_internal(config: &Config) -> Result<()> {
    if let Some(init_target) = &config.init_target {
        return initialize_template_dir(&init_target);
    }
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
