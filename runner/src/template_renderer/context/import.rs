use crate::template_renderer::renderer_config::RendererConfig;
use crate::{util, DisplayNormalized};
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct ImportContext {
    /// Relative (from root) path to the file to import.
    /// e.g. path/to/file_name.ext
    file_path: String,

    /// Name of the file with no extension.
    /// e.g. file_name
    file_name: String,

    /// Name of the file including extension.
    /// e.g. file_name.ext
    file_name_with_ext: String,

    /// Relative (from root) path to the file, interpreted as a package.
    /// This will use the configured package separator.
    ///
    /// ```txt
    ///     root/sub/file_name.ext
    ///     root.sub
    /// ```
    package: String,

    /// Same as `package`, but includes the file name stem.
    ///
    /// ```txt
    ///     root/sub/file_name.ext
    ///     root.sub.file_name
    /// ```
    package_with_file: String,
}

impl ImportContext {
    pub fn new(relative_path: &str, config: &RendererConfig) -> Result<Self> {
        debug!("Creating import context: {}", relative_path);
        let relative_path = PathBuf::from(relative_path);
        let context = Self {
            file_path: relative_path.display_normalized(),
            file_name: util::file_name_or_error(&relative_path.with_extension(""))?,
            file_name_with_ext: util::file_name_or_error(&relative_path)?,
            package: package(&relative_path, &config.package_separator),
            package_with_file: package_with_file(&relative_path, &config.package_separator),
        };
        Ok(context)
    }
}

fn package(path: &Path, separator: &str) -> String {
    let path = match path.parent() {
        None => "".to_string(),
        Some(parent) => parent.display_normalized(),
    };
    path.replace(util::NORMALIZED_SLASH, separator)
}

fn package_with_file(path: &Path, separator: &str) -> String {
    let path = path.with_extension("").display_normalized();
    path.replace(util::NORMALIZED_SLASH, separator)
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::context::import::ImportContext;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;

    #[test]
    fn file_path() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path, &RendererConfig::default())?;
        assert_eq!(context.file_path, path);
        Ok(())
    }

    #[test]
    fn file_name() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path, &RendererConfig::default())?;
        assert_eq!(context.file_name, "file_name");
        Ok(())
    }

    #[test]
    fn file_name_with_ext() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path, &RendererConfig::default())?;
        assert_eq!(context.file_name_with_ext, "file_name.txt");
        Ok(())
    }

    #[test]
    fn package() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path, &RendererConfig::default())?;
        assert_eq!(context.package, "root.sub");
        Ok(())
    }

    #[test]
    fn package_with_file() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path, &RendererConfig::default())?;
        assert_eq!(context.package_with_file, "root.sub.file_name");
        Ok(())
    }

    #[test]
    fn package_with_configured_separator() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let mut config = RendererConfig::default();
        config.package_separator = "::".to_string();
        let context = ImportContext::new(path, &config)?;
        assert_eq!(context.package, "root::sub");
        assert_eq!(context.package_with_file, "root::sub::file_name");
        Ok(())
    }
}
