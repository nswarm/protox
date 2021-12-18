use crate::lang_config::LangConfig;
use crate::Lang;
use anyhow::{anyhow, Context, Result};
use std::fs;

pub fn check_languages_supported(
    name: &str,
    config: &Vec<LangConfig>,
    supported_languages: &[Lang],
) -> Result<()> {
    for lang_config in config {
        if !supported_languages.contains(&lang_config.lang) {
            return Err(anyhow!(
                "Language `{}` is not supported for {} generation.",
                lang_config.lang.as_config(),
                name
            ));
        }
    }
    Ok(())
}

pub(crate) fn create_output_dirs(configs: &Vec<LangConfig>) -> Result<()> {
    for config in configs {
        fs::create_dir_all(&config.output).with_context(|| {
            format!(
                "Failed to create directory at path {:?} for proto output '{}'",
                config.output,
                config.lang.as_config()
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lang_config::LangConfig;
    use crate::run::util::create_output_dirs;
    use crate::Lang;
    use anyhow::Result;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn create_output_dirs_test() -> Result<()> {
        let tempdir = tempdir()?;
        let root = tempdir.path();
        let vec = vec![lang_config_with_output(Lang::Cpp, root)];
        create_output_dirs(&vec)?;
        assert!(fs::read_dir(lang_path(Lang::Cpp, root)).is_ok());
        Ok(())
    }

    fn lang_config_with_output(lang: Lang, root: &Path) -> LangConfig {
        LangConfig {
            lang: lang.clone(),
            output: lang_path(lang, root),
            output_prefix: PathBuf::new(),
        }
    }

    fn lang_path(lang: Lang, root: &Path) -> PathBuf {
        root.join(lang.as_config())
    }
}
