use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

mod config;
mod generator;
mod idl;
mod lang;
mod lang_config;
mod protoc;
mod util;

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
    create_output_root(&config.output_root)?;
    match config.idl {
        Idl::Proto => {
            protoc::generate_descriptor_set_and_builtin_lang_outputs(&config)?;
            generator::generate(&config)?;
        }
    };
    Ok(())
}

fn create_output_root(path: &Path) -> Result<()> {
    Ok(fs::create_dir_all(path).with_context(|| {
        format!(
            "Failed to create output-root directories in path: {}",
            util::normalize_slashes(path.display())
        )
    })?)
}
