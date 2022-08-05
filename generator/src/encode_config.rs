use crate::util;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct EncodeConfig {
    pub target: PathBuf,
    pub message_type: String,
    pub output: PathBuf,
}

impl EncodeConfig {
    pub fn from_config(
        target: &str,
        message_type: &str,
        output: &str,
        output_root: Option<&PathBuf>,
    ) -> Result<Self> {
        Ok(EncodeConfig {
            target: PathBuf::from(target),
            message_type: message_type.to_owned(),
            output: util::path_as_absolute(output, output_root)?,
        })
    }
}
