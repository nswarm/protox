//
// Example usage of idlx in a build.rs script.
//

use generator::{Lang, LangConfig, TemplateConfig};
use std::path::PathBuf;

fn main() {
    // Rerun whenever our input protos change.
    println!("cargo:rerun-if-changed=proto");

    // let input_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // let output_root = PathBuf::from(env!("OUT_DIR"));
    //
    // // Configure idlx directly.
    // let mut config = generator::Config::default();
    // config.input = input_root.join("proto");
    //
    // // Note that these paths need to be absolute. The output_root and template_root config values
    // // are only used when parsing the
    // config.protos.push(LangConfig {
    //     lang: Lang::Rust,
    //     output: output_root.join("proto_rust"),
    // });
    // config.templates.push(TemplateConfig {
    //     input: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("template"),
    //     output: output_root.join("proto_rust"),
    // });
    // generator::generate_with_config(config).unwrap();
}
