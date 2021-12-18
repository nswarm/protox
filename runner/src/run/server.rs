use anyhow::{Result};
use crate::{Config, Lang};
use crate::run::util;

const SUPPORTED_LANGUAGES: [Lang; 0] = [];

pub fn supported_languages() -> &'static [Lang] {
    &SUPPORTED_LANGUAGES
}

pub fn run(config: &Config, _input_files: &Vec<String>) -> Result<()> {
    util::check_languages_supported("server", &config.server, &supported_languages())?;
    Ok(())
}
