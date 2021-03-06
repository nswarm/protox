use crate::{util, Config, Lang};
use anyhow::Result;
use std::path::PathBuf;

pub const SUPPORTED_LANGUAGES: [Lang; 1] = [Lang::Rust];

pub fn generate(config: &Config) -> Result<()> {
    let rust_config = match config
        .protos
        .iter()
        .find(|lang_config| lang_config.lang == Lang::Rust)
    {
        None => return Ok(()),
        Some(config) => config,
    };

    util::check_dir_is_empty(&rust_config.output)?;
    util::create_proto_out_dirs(&[rust_config])?;

    let mut prost_config = prost_build::Config::new();
    // We can skip protoc since we already generate the descriptor fileset with our protoc run.
    prost_config.file_descriptor_set_path(&config.descriptor_set_path);
    prost_config.skip_protoc_run();
    prost_config.out_dir(&rust_config.output);
    for extra_arg in &config.extra_protoc_args {
        prost_config.protoc_arg(util::unquote_arg(extra_arg));
    }
    prost_config.compile_protos(&Vec::<PathBuf>::new(), &[&config.input])?;
    Ok(())
}
