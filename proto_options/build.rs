//
// Generates Extension information for custom protobuf options used during parsing.
//

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=proto_options");
    println!("cargo:rerun-if-changed=build.rs");

    let include_dir = PathBuf::from("protos");
    let input_protos = collect_protos(&include_dir);
    let output_dir = PathBuf::from(env::var("OUT_DIR")?);

    prost_build::Config::default()
        .out_dir(output_dir)
        .compile_protos(&input_protos, &[include_dir])?;

    Ok(())
}

fn collect_protos(dir: &Path) -> Vec<String> {
    WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "proto")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_str().unwrap().to_owned())
        .collect::<Vec<String>>()
}
