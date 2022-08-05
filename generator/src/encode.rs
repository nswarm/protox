use crate::util::DisplayNormalized;
use anyhow::{Context, Result};
use log::info;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::encode_config::EncodeConfig;
use crate::protoc;
use crate::protoc::Protoc;
use crate::Config;

pub fn generate(config: &Config) -> Result<()> {
    let mut protoc = Protoc::new(config)?;
    protoc.add_input_files(
        &mut protoc::input::collect(config).context("Failed to collect input files.")?,
    );
    for encode_config in &config.encode {
        let encode_arg = protoc::arg_with_value("encode", &encode_config.message_type);
        let target_contents = read_target(&encode_config.target)?;
        let output = protoc.execute_with_args(Some(target_contents), &[&encode_arg])?;
        encode_to_file(&output_file_path(encode_config), &output)?;
        log_encode(encode_config);
    }
    Ok(())
}

fn read_target(target: &Path) -> Result<String> {
    let mut target_contents = String::new();
    File::open(target)
        .context("Failed to read encode target.")?
        .read_to_string(&mut target_contents)?;
    Ok(target_contents)
}

fn output_file_path(config: &EncodeConfig) -> PathBuf {
    config
        .output
        .join(config.target.file_stem().unwrap())
        .with_extension("bin")
}

fn encode_to_file(path: &Path, data: &[u8]) -> Result<()> {
    File::create(path)
        .with_context(|| {
            format!(
                "Failed to create encode output file: {:?}",
                path.display_normalized()
            )
        })?
        .write_all(data)
        .with_context(|| {
            format!(
                "Failed to write encoded protobuf to file: {:?}",
                path.display_normalized(),
            )
        })?;
    Ok(())
}

fn log_encode(config: &EncodeConfig) {
    info!(
        "Wrote '{}' encoded as type '{}' to file: {:?}",
        config.target.display_normalized(),
        config.message_type,
        output_file_path(config).display_normalized(),
    )
}
