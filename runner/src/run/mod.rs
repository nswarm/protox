use crate::Config;
use anyhow::{Context, Result};

mod input;
mod proto;

pub fn configured(config: &Config) -> Result<()> {
    let input_files = input::collect(config).context("Failed to collect input files.")?;
    proto::run(config, &input_files)?;
    Ok(())
}
