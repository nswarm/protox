use anyhow::{Context, Result};

pub use crate::protoc::protoc::arg_with_value;
pub use crate::protoc::protoc::Protoc;
use crate::{Config, Lang};

mod builtin;
pub mod input;
mod proto_rust;
mod protoc;

pub fn generate(config: &Config) -> Result<()> {
    if !config.requires_descriptor_set() && config.protos.is_empty() {
        return Ok(());
    }
    let mut protoc = Protoc::new(config)?;
    protoc.add_input_files(&mut input::collect(config).context("Failed to collect input files.")?);
    builtin::register(config, &mut protoc)?;
    protoc.execute(None)?;
    proto_rust::generate(config)?;
    Ok(())
}

pub fn supported_languages() -> Vec<Lang> {
    [
        &builtin::SUPPORTED_LANGUAGES[..],
        &proto_rust::SUPPORTED_LANGUAGES[..],
    ]
    .concat()
}
