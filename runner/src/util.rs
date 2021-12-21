use crate::lang_config::LangConfig;
use crate::util;
use anyhow::{anyhow, Context, Result};
use std::borrow::Borrow;
use std::fs;
use std::path::Path;

pub fn unquote_arg(arg: &str) -> String {
    arg[1..arg.len() - 1].to_string()
}

pub fn create_output_dirs<C: Borrow<LangConfig>>(configs: &[C]) -> Result<()> {
    for config in configs {
        let config = config.borrow();
        fs::create_dir_all(&config.output).with_context(|| {
            format!(
                "Failed to create directory at path {:?} for output '{}'",
                config.output,
                config.lang.as_config()
            )
        })?;
    }
    Ok(())
}

pub fn create_dir_or_error(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| {
        format!(
            "Failed to create directories for path {}",
            path.display_normalized()
        )
    })
}

/// Creates the file including any necessary directories.
pub fn create_file_or_error(path: &Path) -> Result<fs::File> {
    match path.parent() {
        None => {}
        Some(parent) => util::create_dir_or_error(parent)?,
    }
    fs::File::create(&path).with_context(|| {
        format!(
            "Failed to create file at path {}",
            path.display_normalized()
        )
    })
}

pub fn str_or_error<F: Fn() -> String>(value: &Option<String>, error: F) -> Result<&str> {
    let result = value
        .as_ref()
        .map(String::as_str)
        .ok_or(anyhow!("{}", error()))?;
    Ok(result)
}

pub fn str_or_unknown(str: &Option<String>) -> &str {
    static UNKNOWN: &str = "(unknown)";
    str.as_ref().map(|s| s.as_str()).unwrap_or(&UNKNOWN)
}

pub fn normalize_slashes(path: impl ToString) -> String {
    path.to_string().replace("\\", "/")
}

pub trait DisplayNormalized {
    fn display_normalized(&self) -> String;
}

impl DisplayNormalized for Path {
    fn display_normalized(&self) -> String {
        normalize_slashes(self.display())
    }
}

#[cfg(test)]
mod tests {
    use crate::lang_config::LangConfig;
    use crate::util::{create_output_dirs, DisplayNormalized};
    use crate::Lang;
    use anyhow::Result;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn normalized_path_display() {
        let path = PathBuf::from("/test\\path\\display/slashes");
        assert_eq!(
            path.display_normalized(),
            "/test/path/display/slashes".to_string()
        );
    }

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
