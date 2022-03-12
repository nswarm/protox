use crate::lang_config::LangConfig;
use crate::{extension_registry, Config};
use anyhow::{anyhow, Context, Result};
use prost::Message;
use prost_types::FileDescriptorSet;
use std::borrow::Borrow;
use std::fs;
use std::path::{Path, PathBuf};

pub fn unquote_arg(arg: &str) -> String {
    arg[1..arg.len() - 1].to_string()
}

pub(crate) fn check_dir_is_empty(dir: &Path) -> Result<()> {
    if dir.exists() && fs::read_dir(dir)?.count() > 0 {
        Err(anyhow!(
            "Target directory '{}' is not empty.",
            dir.display_normalized()
        ))
    } else {
        Ok(())
    }
}

pub fn create_proto_out_dirs<C: Borrow<LangConfig>>(configs: &[C]) -> Result<()> {
    for config in configs {
        let config = config.borrow();
        fs::create_dir_all(&config.output).with_context(|| {
            format!(
                "Failed to create directory at path {} for output '{}'",
                config.output.display_normalized(),
                config.lang.as_config(),
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
        Some(parent) => fs::create_dir_all(parent)?,
    }
    fs::File::create(&path).with_context(|| {
        format!(
            "Failed to create file at path '{}'",
            path.display_normalized()
        )
    })
}

pub fn path_parent_or_error(path: &Path) -> Result<&Path> {
    path.parent().ok_or(anyhow!(
        "File path has no parent: '{}'.",
        path.display_normalized()
    ))
}

pub fn file_name_or_error(path: &Path) -> Result<String> {
    let result = path
        .file_name()
        .ok_or(anyhow!(
            "File path has no file name: '{}'.",
            path.display_normalized()
        ))?
        .to_str()
        .ok_or(anyhow!(
            "File path is not unicode: '{}'",
            path.display_normalized()
        ))?
        .to_string();
    Ok(result)
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

pub const NORMALIZED_SLASH: &str = "/";
pub fn normalize_slashes(path: impl ToString) -> String {
    path.to_string().replace("\\", NORMALIZED_SLASH)
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

/// Returns the path itself if it is absolute, or joined to `root` if not.
pub fn path_as_absolute<P: AsRef<Path>>(
    path_str: &str,
    default_root: Option<&P>,
) -> Result<PathBuf> {
    let path = PathBuf::from(path_str);
    if path.is_absolute() {
        return Ok(path);
    }
    match default_root {
        None => Err(anyhow!(
            "Path {} is relative but no root was specified.",
            path_str,
        )),
        Some(root) => Ok(root.as_ref().join(path)),
    }
}

pub(crate) fn load_descriptor_set(config: &Config) -> Result<FileDescriptorSet> {
    let path = &config.descriptor_set_path;
    let bytes = fs::read(&path).with_context(|| {
        format!(
            "Failed to read file descriptor set at path: {}",
            path.display_normalized()
        )
    })?;
    let descriptor_set =
        Message::decode_with_extensions(bytes.as_slice(), extension_registry::create())?;
    Ok(descriptor_set)
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
    use crate::util::{create_proto_out_dirs, DisplayNormalized};
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
        create_proto_out_dirs(&vec[..])?;
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

    mod path_as_absolute {
        use crate::util::path_as_absolute;
        use crate::DisplayNormalized;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn absolute_with_root() -> Result<()> {
            let root = env::current_dir()?;
            let path = env::temp_dir();
            let result = path_as_absolute(&path.display_normalized(), Some(&root))?;
            assert_eq!(result, path);
            Ok(())
        }

        #[test]
        fn relative_with_root() -> Result<()> {
            let root = env::current_dir()?;
            let path = "relative/path";
            let result = path_as_absolute(path, Some(&root))?;
            assert_eq!(result, root.join(path));
            Ok(())
        }

        #[test]
        fn absolute_without_root() -> Result<()> {
            let path = env::temp_dir();
            let result = path_as_absolute::<PathBuf>(&path.display_normalized(), None)?;
            assert_eq!(result, path);
            Ok(())
        }

        #[test]
        fn relative_without_root() -> Result<()> {
            let path = "relative/path";
            assert!(path_as_absolute::<PathBuf>(path, None).is_err());
            Ok(())
        }
    }
}
