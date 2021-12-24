use crate::lang::Lang;
use crate::util;
use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;

pub struct LangConfig {
    pub lang: Lang,
    pub output: PathBuf,
}

impl LangConfig {
    pub fn from_config(lang: &str, output: &str, output_root: Option<&PathBuf>) -> Result<Self> {
        let output_path = util::parse_rooted_path(output_root, output, "output")?;
        Ok(LangConfig {
            lang: Lang::from_str(lang)?,
            output: output_path,
        })
    }
}

// todo do I need this?
impl AsRef<LangConfig> for LangConfig {
    fn as_ref(&self) -> &LangConfig {
        &self
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::Lang;
    use crate::lang_config::LangConfig;
    use crate::DisplayNormalized;
    use anyhow::Result;
    use std::env::current_dir;

    #[test]
    fn from_config_with_explicit_output() -> Result<()> {
        let output_path = current_dir()?;
        let config = LangConfig::from_config(
            &Lang::CSharp.as_config(),
            &output_path.display_normalized(),
            None,
        )?;
        assert_eq!(config.lang, Lang::CSharp);
        assert_eq!(config.output, output_path);
        Ok(())
    }

    #[test]
    fn from_config_with_unsupported_lang() -> Result<()> {
        assert!(LangConfig::from_config(
            "blah unsupported lang",
            &current_dir()?.display_normalized(),
            None
        )
        .is_err());
        Ok(())
    }
}
