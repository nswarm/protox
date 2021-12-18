use anyhow::{anyhow, Result};
use crate::Lang;
use crate::lang_config::LangConfig;

pub fn check_languages_supported(name: &str, config: &Vec<LangConfig>, supported_languages: &[Lang]) -> Result<()> {
    for lang_config in config {
        if !supported_languages.contains(&lang_config.lang) {
            return Err(anyhow!("Language `{}` is not supported for {} generation.", lang_config.lang.as_config(), name));
        }
    }
    Ok(())
}
