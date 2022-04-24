//
// Example usage of protox in a build.rs script.
//

use generator::{InOutConfig, Lang, LangConfig};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    // Rerun whenever our input protos change.
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rerun-if-changed=build.rs");

    let module_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_dir = module_root.join("input");
    let protox_includes_dir = module_root.join("../proto_options/protos");
    let output_dir = PathBuf::from(env::var("OUT_DIR")?);
    let proto_out = output_dir.join("rust-proto");
    let template_out = output_dir.join("rust-example");

    // Clear target dirs. (protox guards against creating output in non-empty directories.)
    clean_dir(&proto_out)?;
    clean_dir(&template_out)?;

    // Configure protox directly.
    let mut config = generator::Config::default();
    config.input = input_dir.join("proto");
    config.includes = vec![protox_includes_dir.to_str().unwrap().to_owned()];
    config.descriptor_set_path = output_dir.join("descriptor_set");

    // Note that these paths need to be absolute.
    config.protos.push(LangConfig {
        lang: Lang::Rust,
        output: proto_out,
    });
    config.templates.push(InOutConfig {
        input: input_dir.join("templates").join("rust-example"),
        output: template_out,
    });

    std::env::set_var("RUST_LOG", "info,handlebars=off");
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
