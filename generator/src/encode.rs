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
        File::create(&output_file_path(encode_config))
            .context("Failed to create encode output file.")?
            .write_all(&output)
            .context("Failed to write encoded protobuf.")?;
        info!(
            "Wrote '{}' encoded as type '{}' to file: {:?}",
            encode_config.target.display_normalized(),
            encode_config.message_type,
            output_file_path(encode_config).display_normalized(),
        )
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
