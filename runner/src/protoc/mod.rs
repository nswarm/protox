use anyhow::{Context, Result};

pub use proto::SUPPORTED_LANGUAGES;

use crate::protoc::protoc::Protoc;
use crate::Config;

mod input;
mod proto;
mod protoc;

pub fn generate_descriptor_set_and_builtin_lang_outputs(config: &Config) -> Result<()> {
    let mut protoc = Protoc::new(config)?;
    protoc.add_input_files(&mut input::collect(config).context("Failed to collect input files.")?);
    proto::register(config, &mut protoc)?;
    protoc.execute()?;
    Ok(())
}
