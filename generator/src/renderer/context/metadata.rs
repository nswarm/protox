use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{util, DisplayNormalized};

pub type PackageTree = HashMap<String, PackageTreeNode>;

#[derive(Serialize, Deserialize, Clone)]
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
    ///     root: root.rs
    ///     root.sub: root-sub.rs
    ///     root.sub.inner: root-sub-inner.rs
    /// ```
    package_files_full: Vec<PackageFile>,

    /// When one_file_per_package, this is a tree of relative package keys -> files.
    /// Using the {{> file_name}} handlebars operator, this data can be recursively iterated and
    /// printed to a file.
    ///
    /// metadata.hbs
    /// ```hbs
    /// {{~#each package_file_tree}}
    /// {{> package_tree_node}}
    /// {{/each}}
    /// ```
    ///
    /// package_tree_node.hbs
    /// ```hbs
    /// {{@key}}: {
    /// {{#if file_name}}
    ///     file_name: "{{file_name}}"
    /// {{/if}}
    /// {{#each children}}
    ///     {{> package_tree_node}}
    /// {{/each}}
    /// }
    /// ```
    ///
    /// Result:
    /// ```txt
    ///
    ///     root: {
    ///         file_name: root.rs
    ///         children: {
    ///             sub: {
    ///                 file_name: root-sub.rs
    ///                 children: { ...etc }
    ///             }
    ///             sub2: { ...etc }
    ///         }
    ///     }
    ///     other: { ...etc }
    ///```
    ///
    /// Note: Currently indentation does not work for partials.
    ///
    package_file_tree: PackageTree,

    #[serde(skip)]
    directory_path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
struct PackageFile {
    package: String,
    file_name: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PackageTreeNode {
    file_name: Option<String>,
    children: PackageTree,
}

impl MetadataContext {
    pub fn new() -> Self {
        Self {
            directory: "".to_owned(),
            file_names: vec![],
            file_names_with_ext: vec![],
            subdirectories: vec![],
            package_files_full: vec![],
            package_file_tree: Default::default(),
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
                .to_owned(),
            file_names: vec![],
            file_names_with_ext: vec![],
            subdirectories: vec![],
            package_files_full: vec![],
            package_file_tree: Default::default(),
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
        self.package_file_tree = create_package_file_tree(&package_files);
        self.package_files_full = package_files
            .into_iter()
            .map(|(package, path)| PackageFile {
                package: package.to_owned(),
                file_name: path.as_ref().display_normalized(),
            })
            .collect::<Vec<PackageFile>>();
    }

    fn is_direct_child(&self, path: &Path) -> bool {
        match path.parent() {
            None => self.directory_path.as_os_str().is_empty(),
            Some(parent) => parent == self.directory_path,
        }
    }
}

/// Converts a map of fully-qualified package -> file name to a tree of package components that
/// include the associated file path.
fn create_package_file_tree(package_files: &HashMap<String, impl AsRef<Path>>) -> PackageTree {
    let mut tree = PackageTree::new();
    for (package, file_name) in package_files {
        let mut package_it = &mut tree;
        let components = package.split('.');
        let components_len = components.clone().count();
        for (i, component) in components.enumerate() {
            let node = package_it
                .entry(component.to_owned())
                .or_insert_with(|| PackageTreeNode::default());
            if i == components_len - 1 {
                node.file_name = Some(file_name.as_ref().display_normalized());
            }
            package_it = &mut node.children;
        }
    }
    tree
}

#[cfg(test)]
mod tests {
    mod push_file {
        use std::path::PathBuf;

        use anyhow::Result;

        use crate::renderer::context::MetadataContext;

        #[test]
        fn direct_child() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_file(&root.join("file.txt"))?;
            assert_eq!(context.file_names.get(0), Some(&"file".to_owned()));
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
            assert_eq!(context.file_names.get(0), Some(&"file".to_owned()));
            assert_eq!(
                context.file_names_with_ext.get(0),
                Some(&"file.txt".to_owned())
            );
            Ok(())
        }
    }

    mod push_subdirectory {
        use std::path::PathBuf;

        use anyhow::Result;

        use crate::renderer::context::MetadataContext;

        #[test]
        fn direct_child() -> Result<()> {
            let root = PathBuf::from("root");
            let mut context = MetadataContext::with_relative_dir(&root)?;
            context.push_subdirectory(&root.join("sub"))?;
            assert_eq!(context.subdirectories.get(0), Some(&"sub".to_owned()));
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
        use std::path::PathBuf;

        use anyhow::Result;

        use crate::renderer::context::MetadataContext;

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

    mod create_package_file_tree {
        use std::collections::HashMap;
        use std::path::PathBuf;

        use anyhow::{anyhow, Result};

        use crate::renderer::context::metadata::{
            create_package_file_tree, PackageTree, PackageTreeNode,
        };

        #[test]
        fn separate_top_level() -> Result<()> {
            let package_files = create_package_file_map(&[
                ("root", "file0"),
                ("other", "file1"),
                ("third", "file2"),
            ]);
            let tree = create_package_file_tree(&package_files);
            assert_tree_node_file(&tree, "root", Some("file0"))?;
            assert_tree_node_file(&tree, "other", Some("file1"))?;
            assert_tree_node_file(&tree, "third", Some("file2"))?;
            Ok(())
        }

        #[test]
        fn deep_file() -> Result<()> {
            let package_files = create_package_file_map(&[("root.sub.inner.sanctum", "file")]);
            let tree = create_package_file_tree(&package_files);
            assert_tree_node_file(&tree, "root", None)?;
            assert_tree_node_file(&tree, "root.sub", None)?;
            assert_tree_node_file(&tree, "root.sub.inner", None)?;
            assert_tree_node_file(&tree, "root.sub.inner.sanctum", Some("file"))?;
            Ok(())
        }

        fn create_package_file_map(values: &[(&str, &str)]) -> HashMap<String, PathBuf> {
            let mut package_files = HashMap::new();
            for (package, file) in values {
                package_files.insert(package.to_string(), PathBuf::from(file));
            }
            package_files
        }

        fn assert_tree_node_file(
            tree: &PackageTree,
            package: &str,
            file_name: Option<&str>,
        ) -> Result<()> {
            let root_node = &PackageTreeNode {
                file_name: None,
                children: tree.clone(),
            };
            let mut node = root_node;
            for component in package.split('.') {
                node = node
                    .children
                    .get(component)
                    .ok_or(anyhow!("Expected tree to have component: {}", package))?;
            }
            assert_eq!(node.file_name, file_name.map(str::to_owned));
            Ok(())
        }
    }
}
