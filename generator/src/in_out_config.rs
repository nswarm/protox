use crate::util;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct InOutConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub overlays: Vec<PathBuf>,
}

impl InOutConfig {
    pub fn from_config(
        input: &str,
        output: &str,
        input_root: Option<&PathBuf>,
        output_root: Option<&PathBuf>,
    ) -> Result<Self> {
        Ok(InOutConfig {
            input: util::path_as_absolute(input, input_root)?,
            output: util::path_as_absolute(output, output_root)?,
            // Only used when converting from a more specific config.
            overlays: vec![],
        })
    }
}
