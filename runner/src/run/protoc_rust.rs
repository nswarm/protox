use crate::lang::Lang;
use crate::run::util;
use crate::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Special case since rust uses prost plugin.
pub fn run(config: &Config, input_files: &Vec<String>) -> Result<()> {
    let rust_config = config
        .proto
        .iter()
        .find(|lang_config| lang_config.lang == Lang::Rust);
    let output = match rust_config {
        None => return Ok(()),
        Some(rust_config) => &rust_config.output,
    };

    create_output_dir(output)?;

    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir(output);
    for extra_arg in &config.extra_protoc_args {
        prost_config.protoc_arg(util::unquote_arg(extra_arg));
    }
    prost_config.compile_protos(input_files, &[&config.input])?;
    Ok(())
}

fn create_output_dir(output: &Path) -> Result<()> {
    fs::create_dir_all(&output).with_context(|| {
        format!(
            "Failed to create directory at path {:?} for proto output '{}'",
            output,
            Lang::Rust.as_config(),
        )
    })?;
    Ok(())
}
