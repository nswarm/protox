use crate::Config;
use anyhow::Result;

/// Special case since rust uses prost plugin.
pub fn run(config: &Config, input_files: &Vec<String>) -> Result<()> {
    // todo uhh ok I think normal protoc just ignres and this does prost only
    // prost_build::compile_protos()
    Ok(())
}
