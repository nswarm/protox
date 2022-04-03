use std::path::PathBuf;

use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{util, DisplayNormalized};

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
}

impl ImportContext {
    pub fn new(relative_path: &str) -> Result<Self> {
        debug!("Creating import context: {}", relative_path);
        let relative_path = PathBuf::from(relative_path);
        let context = Self {
            file_path: relative_path.display_normalized(),
            file_name: util::file_name_or_error(&relative_path.with_extension(""))?,
            file_name_with_ext: util::file_name_or_error(&relative_path)?,
        };
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::renderer::context::import::ImportContext;

    #[test]
    fn file_path() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path)?;
        assert_eq!(context.file_path, path);
        Ok(())
    }

    #[test]
    fn file_name() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path)?;
        assert_eq!(context.file_name, "file_name");
        Ok(())
    }

    #[test]
    fn file_name_with_ext() -> Result<()> {
        let path = "root/sub/file_name.txt";
        let context = ImportContext::new(path)?;
        assert_eq!(context.file_name_with_ext, "file_name.txt");
        Ok(())
    }
}
