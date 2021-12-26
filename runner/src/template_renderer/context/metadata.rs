use crate::{util, DisplayNormalized};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct MetadataContext {
    /// Relative path to this directory.
    directory: String,

    /// Names of files in this directory, without extensions.
    file_names: Vec<String>,

    /// Names of tiles in this directory, with extensions.
    file_names_with_ext: Vec<String>,

    /// Names of directories in this directory.
    subdirectories: Vec<String>,

    /// When one_file_per_package is enabled, this list holds the package->file mapping.
    /// Each package is fully specified.
    ///
    /// ```txt
    ///     root -> root.rs
    ///     root.sub -> root-sub.rs
    ///     root.sub.inner -> root-sub-inner.rs
    /// ```
    package_files_full: Vec<PackageFile>,

    #[serde(skip)]
    directory_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct PackageFile {
    package: String,
    file_name: String,
}

impl MetadataContext {
    pub fn new() -> Self {
        Self {
            directory: "".to_string(),
            file_names: vec![],
            file_names_with_ext: vec![],
            subdirectories: vec![],
            package_files_full: vec![],
            directory_path: Default::default(),
        }
    }

    pub fn with_relative_dir(directory: &Path) -> Result<Self> {
        let context = Self {
            directory: directory
                .to_str()
                .ok_or(anyhow!(
                    "Cannot create MetadataContext, dir path is not valid: {:?}",
                    directory
                ))?
                .to_string(),
            file_names: vec![],
            file_names_with_ext: vec![],
            subdirectories: vec![],
            package_files_full: vec![],
            directory_path: directory.to_path_buf(),
        };
        Ok(context)
    }

    pub fn relative_dir(&self) -> &Path {
        &self.directory_path
    }

    pub fn push_file(&mut self, path: &Path) -> Result<()> {
        if self.is_direct_child(path) {
            let file_name_with_ext = util::file_name_or_error(path)?;
            self.file_names_with_ext.push(file_name_with_ext);
            let file_name_no_ext = util::file_name_or_error(&path.with_extension(""))?;
            self.file_names.push(file_name_no_ext);
        }
        Ok(())
    }

    pub fn push_subdirectory(&mut self, path: &Path) -> Result<()> {
        if path.as_os_str().is_empty() {
            return Ok(());
        }
        if self.is_direct_child(path) {
            let dir_name = util::file_name_or_error(path)?;
            self.subdirectories.push(dir_name);
        }
        Ok(())
    }

    pub fn append_files(&mut self, paths: &[impl AsRef<Path>]) -> Result<()> {
        for path in paths {
            self.push_file(path.as_ref())?;
        }
        Ok(())
    }

    pub fn append_subdirectories<I, T>(&mut self, paths: I) -> Result<()>
    where
        I: Iterator<Item = T>,
        T: AsRef<Path>,
    {
        for path in paths {
            self.push_subdirectory(path.as_ref())?;
        }
        Ok(())
    }

    pub fn append_package_files(&mut self, package_files: HashMap<String, impl AsRef<Path>>) {
        self.package_files_full = package_files
            .into_iter()
            .map(|(package, path)| PackageFile {
                package: package.to_string(),
                file_name: path.as_ref().display_normalized(),
            })
            .collect::<Vec<PackageFile>>()
    }

    fn is_direct_child(&self, path: &Path) -> bool {
        match path.parent() {
            None => self.directory_path.as_os_str().is_empty(),
            Some(parent) => parent == self.directory_path,
        }
    }
}

#[cfg(test)]
mod tests {
    mod push_file {
        use crate::template_renderer::context::MetadataContext;
        use anyhow::Result;
        use std::path::PathBuf;

        #[test]
        fn direct_child() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_file(&root.join("file.txt"))?;
            assert_eq!(context.file_names.get(0), Some(&"file".to_string()));
            Ok(())
        }

        #[test]
        fn not_direct_child() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_file(&root.join("sub/file.txt"))?;
            assert!(context.file_names.is_empty());
            Ok(())
        }

        #[test]
        fn also_adds_no_ext_file_name() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_file(&root.join("file.txt"))?;
            assert_eq!(context.file_names.get(0), Some(&"file".to_string()));
            assert_eq!(
                context.file_names_with_ext.get(0),
                Some(&"file.txt".to_string())
            );
            Ok(())
        }
    }

    mod push_subdirectory {
        use crate::template_renderer::context::MetadataContext;
        use anyhow::Result;
        use std::path::PathBuf;

        #[test]
        fn direct_child() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_subdirectory(&root.join("sub"))?;
            assert_eq!(context.subdirectories.get(0), Some(&"sub".to_string()));
            Ok(())
        }

        #[test]
        fn not_direct_child() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_subdirectory(&root.join("sub/other"))?;
            assert!(context.subdirectories.is_empty());
            Ok(())
        }
    }

    mod direct_child {
        use crate::template_renderer::context::MetadataContext;
        use anyhow::Result;
        use std::path::PathBuf;

        #[test]
        fn valid() -> Result<()> {
            let root = PathBuf::from("root");
            let context = MetadataContext::with_relative_dir(&root)?;
            assert!(context.is_direct_child(&root.join("anything")));
            Ok(())
        }

        #[test]
        fn invalid_too_deep() -> Result<()> {
            let root = PathBuf::from("root");
            let context = MetadataContext::with_relative_dir(&root)?;
            assert!(!context.is_direct_child(&root.join("sub/other")));
            Ok(())
        }

        #[test]
        fn invalid_too_high() -> Result<()> {
            let root = PathBuf::from("root/sub");
            let context = MetadataContext::with_relative_dir(&root)?;
            assert!(!context.is_direct_child(&PathBuf::from("root")));
            Ok(())
        }

        #[test]
        fn invalid_different_root() -> Result<()> {
            let root = PathBuf::from("root/sub");
            let context = MetadataContext::with_relative_dir(&root)?;
            assert!(!context.is_direct_child(&PathBuf::from("anything")));
            Ok(())
        }
    }
}
