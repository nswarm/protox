use crate::run::protoc::Protoc;
use crate::run::util;
use crate::{Config, Lang};
use anyhow::Result;

const SUPPORTED_LANGUAGES: [Lang; 0] = [];

pub fn supported_languages() -> &'static [Lang] {
    &SUPPORTED_LANGUAGES
}

pub fn run(config: &Config, protoc: &mut Protoc) -> Result<()> {
    if config.client.is_empty() {
        return Ok(());
    }
    util::check_languages_supported("client", &config.client, &supported_languages())?;
    util::create_output_dirs(&config.client)?;
    protoc.flag_for_execution();
    Ok(())
}
