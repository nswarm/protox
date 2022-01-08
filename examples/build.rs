//
// Example usage of idlx in a build.rs script.
//

use generator::{Lang, LangConfig, TemplateConfig};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    // Rerun whenever our input protos change.
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rerun-if-changed=build.rs");

    let input_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("input");
    let output_dir = PathBuf::from(env::var("OUT_DIR")?);
    let proto_out = output_dir.join("rust-proto");
    let template_out = output_dir.join("rust-server");

    // Clear target dirs. (idlx guards against creating output in non-empty directories.)
    clean_dir(&proto_out)?;
    clean_dir(&template_out)?;

    // Configure idlx directly.
    let mut config = generator::Config::default();
    config.input = input_dir.join("proto");
    config.descriptor_set_path = output_dir.join("descriptor_set");

    // Note that these paths need to be absolute.
    config.protos.push(LangConfig {
        lang: Lang::Rust,
        output: proto_out,
    });
    config.templates.push(TemplateConfig {
        input: input_dir.join("templates").join("rust-server"),
        output: template_out,
    });
    generator::generate_with_config(config)?;

    Ok(())
}

fn clean_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    // Check first so we don't error if path doesn't exist.
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    Ok(())
}
