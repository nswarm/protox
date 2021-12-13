use crate::run::{protoc_basic, protoc_rust};
use crate::Config;
use anyhow::{anyhow, Context, Result};
use log::debug;
use std::path::Path;
use walkdir::WalkDir;

pub fn run(config: &Config) -> Result<()> {
    let input_files = collect_input_files(config).context("Failed to collect input files.")?;
    protoc_basic::run(config, &input_files)?;
    protoc_rust::run(config, &input_files)?;
    Ok(())
}

fn collect_input_files(config: &Config) -> Result<Vec<String>> {
    let mut inputs = Vec::new();
    for entry in WalkDir::new(&config.input).follow_links(false).into_iter() {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        if !is_proto_ext(entry.path()) {
            continue;
        }
        debug!("collect_inputs found proto file: {:?}", entry.path());
        let input = entry
            .path()
            .strip_prefix(&config.input)?
            .to_str()
            .ok_or(anyhow!("Failed to convert path to str: {:?}", entry.path()))?
            .to_string();
        inputs.push(normalize_slashes(&input));
    }
    Ok(inputs)
}

fn is_proto_ext(path: &Path) -> bool {
    match path.extension() {
        Some(ext) if ext == "proto" => true,
        _ => false,
    }
}

fn normalize_slashes(path: &str) -> String {
    path.replace("\\", "/")
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use std::path::Path;

    mod collect_input_files {
        use crate::run::protoc::collect_input_files;
        use crate::run::protoc::tests::config_with_input;
        use anyhow::Result;
        use std::fs;
        use std::path::{Path, PathBuf};
        use tempfile::tempdir;

        #[test]
        fn collects_all_in_dir() -> Result<()> {
            let dir = tempdir()?;
            let root = dir.path();
            create_files_at(root, &["aaa.proto", "bbb.proto", "ccc.proto"])?;
            let files = collect_input_files(&config_with_input(root))?;
            assert_eq!(files.len(), 3);
            Ok(())
        }

        #[test]
        fn collects_all_recursively() -> Result<()> {
            let dir = tempdir()?;
            let root = dir.path();
            create_files_at(
                root,
                &[
                    "aaa.proto",
                    "a/aaa.proto",
                    "a/bbb.proto",
                    "a/b/aaa.proto",
                    "a/b/bbb.proto",
                    "a/b/c/aaa.proto",
                ],
            )?;
            let files = collect_input_files(&config_with_input(root))?;
            assert_eq!(files.len(), 6);
            Ok(())
        }

        #[test]
        fn paths_are_absolute() -> Result<()> {
            let dir = tempdir()?;
            let root = dir.path();

            create_files_at(root, &["aaa.proto"])?;
            let files = collect_input_files(&config_with_input(root))?;
            assert_arg_equal_to_path(files.get(0).unwrap(), "aaa.proto");

            create_files_at(root, &["a/b/c/aaa.proto"])?;
            let files = collect_input_files(&config_with_input(root))?;
            assert_arg_equal_to_path(files.get(0).unwrap(), "a/b/c/aaa.proto");

            Ok(())
        }

        #[test]
        fn ignores_non_proto() -> Result<()> {
            let dir = tempdir()?;
            let root = dir.path();
            create_files_at(
                root,
                &[
                    "aaa.proto",
                    "aaa.txt",
                    "aaap.roto",
                    "aaa",
                    "a/aaa.txt",
                    "a/b/aaa.txt",
                ],
            )?;
            let files = collect_input_files(&config_with_input(root))?;
            assert_arg_equal_to_path(files.get(0).unwrap(), "aaa.proto");
            Ok(())
        }

        fn assert_arg_equal_to_path(arg: &str, path: &str) {
            assert_eq!(PathBuf::from(arg).as_path(), PathBuf::from(path));
        }

        fn create_files_at(root: &Path, paths: &[&str]) -> Result<()> {
            for path in paths {
                fs::create_dir_all(root.join(path).parent().unwrap())?;
                fs::write(root.join(path), "arbitrary")?;
            }
            Ok(())
        }
    }

    mod is_proto_ext {
        use crate::run::protoc::is_proto_ext;
        use std::path::PathBuf;

        #[test]
        fn valid() {
            let path = PathBuf::from("path/to/file.proto");
            assert!(is_proto_ext(&path));
        }

        #[test]
        fn no_ext() {
            let path = PathBuf::from("path/to/file");
            assert!(!is_proto_ext(&path));
        }

        #[test]
        fn different_ext() {
            let path = PathBuf::from("path/to/filep.roto");
            assert!(!is_proto_ext(&path));
        }
    }

    fn config_with_input(path: &Path) -> Config {
        let mut config = Config::default();
        config.input = path.to_path_buf();
        config
    }
}
