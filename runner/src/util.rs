use crate::lang_config::LangConfig;
use anyhow::{Context, Result};
use std::borrow::Borrow;
use std::fs;

pub fn unquote_arg(arg: &str) -> String {
    arg[1..arg.len() - 1].to_string()
}

pub fn create_output_dirs<C: Borrow<LangConfig>>(configs: &[C]) -> Result<()> {
    for config in configs {
        let config = config.borrow();
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
    use crate::util::create_output_dirs;
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
        create_output_dirs(&vec[..])?;
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
