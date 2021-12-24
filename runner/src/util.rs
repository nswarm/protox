use crate::lang_config::LangConfig;
use crate::util;
use anyhow::{anyhow, Context, Result};
use std::borrow::Borrow;
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn replace_proto_ext(file_name: &str, new_ext: &str) -> String {
    if new_ext.starts_with(".") {
        static PROTO_EXT_FULL: &str = ".proto";
        file_name.replace(PROTO_EXT_FULL, new_ext)
    } else {
        static PROTO_EXT_RAW: &str = "proto";
        file_name.replace(PROTO_EXT_RAW, new_ext)
    }
}

pub(crate) fn parse_rooted_path<P: AsRef<Path>>(
    root: Option<&P>,
    path_str: &str,
    root_arg_name: &str,
) -> Result<PathBuf> {
    let path = PathBuf::from(path_str);
    if path.is_absolute() {
        return Ok(path);
    }
    match root {
        None => Err(anyhow!(
            "Path {} is relative but no {} root was specified.",
            path_str,
            root_arg_name,
        )),
        Some(root) => Ok(root.as_ref().join(path)),
    }
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
        let vec = vec![
            lang_config_with_output(Lang::Cpp, root),
            lang_config_with_output(Lang::CSharp, root),
        ];
        create_output_dirs(&vec[..])?;
        assert!(fs::read_dir(root.join(Lang::Cpp.as_config())).is_ok());
        assert!(fs::read_dir(root.join(Lang::CSharp.as_config())).is_ok());
        Ok(())
    }

    fn lang_config_with_output(lang: Lang, root: &Path) -> LangConfig {
        LangConfig {
            lang: lang.clone(),
            output: root.join(lang.as_config()),
        }
    }

    mod parse_rooted_path {
        use crate::util::parse_rooted_path;
        use crate::DisplayNormalized;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn absolute_with_root() -> Result<()> {
            let root = env::current_dir()?;
            let path = env::temp_dir();
            let result = parse_rooted_path(Some(&root), &path.display_normalized(), "test")?;
            assert_eq!(result, path);
            Ok(())
        }

        #[test]
        fn relative_with_root() -> Result<()> {
            let root = env::current_dir()?;
            let path = "relative/path";
            let result = parse_rooted_path(Some(&root), path, "test")?;
            assert_eq!(result, root.join(path));
            Ok(())
        }

        #[test]
        fn absolute_without_root() -> Result<()> {
            let path = env::temp_dir();
            let result = parse_rooted_path::<PathBuf>(None, &path.display_normalized(), "test")?;
            assert_eq!(result, path);
            Ok(())
        }

        #[test]
        fn relative_without_root() -> Result<()> {
            let path = "relative/path";
            assert!(parse_rooted_path::<PathBuf>(None, path, "test").is_err());
            Ok(())
        }
    }
}
