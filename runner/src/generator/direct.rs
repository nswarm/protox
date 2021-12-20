use crate::{util, Config, Lang};
use anyhow::Result;

pub const SUPPORTED_LANGUAGES: [Lang; 0] = [];

pub fn generate(config: &Config) -> Result<()> {
    if config.direct.is_empty() {
        return Ok(());
    }
    util::create_output_dirs(&config.direct)?;
    Ok(())
}
