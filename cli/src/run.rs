use crate::Options;
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const PROTOC_ARG_PROTO_PATH: &str = "proto_path";

pub fn protoc(options: &Options) -> Result<()> {
    let protoc_path = protoc_path();
    let args = collect_and_validate_args(options)?;

    info!("using protoc at path: {:?}", protoc_path);
    info!("running command:\tprotoc {}", args.join(" "));

    create_output_paths(options)?;

    let mut child = Command::new(protoc_path)
        .args(args)
        .spawn()
        .context("Failed to execute spawn protoc process.")?;
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Exited with status {}", status))
    }
}

fn collect_and_validate_args(options: &Options) -> Result<Vec<String>> {
    let mut args = Vec::new();
    collect_proto_path(options, &mut args).context("Failed to collect proto_path arg.")?;
    collect_proto_outputs(options, &mut args).context("Failed to collect proto output args.")?;
    // Input files must always come last.
    for file_path in collect_input_files(options).context("Failed to collect input files.")? {
        args.push(file_path)
    }
    Ok(args)
}

fn collect_proto_path(options: &Options, args: &mut Vec<String>) -> Result<()> {
    if let Err(_) = fs::read_dir(&options.input) {
        bail!(
            "Invalid input: could not find the directory located at path '{:?}'.",
            options.input
        );
    }
    let input = match options.input.to_str() {
        None => bail!("Invalid input: Could not parse path to string."),
        Some(input) => input,
    };
    args.push(arg_with_value(PROTOC_ARG_PROTO_PATH, input));
    Ok(())
}

fn collect_proto_outputs(options: &Options, args: &mut Vec<String>) -> Result<()> {
    for option in &options.proto {
        let arg = [option.lang.as_config().as_str(), "_out"].concat();
        let value = option
            .output
            .to_str()
            .ok_or(anyhow!("Output path is invalid: {:?}", option.output))?;
        args.push(arg_with_value(&arg, value));
    }
    Ok(())
}

fn collect_input_files(options: &Options) -> Result<Vec<String>> {
    let mut inputs = Vec::new();
    for entry in WalkDir::new(&options.input).follow_links(false).into_iter() {
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
            .strip_prefix(&options.input)?
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

fn create_output_paths(options: &Options) -> Result<()> {
    for proto in &options.proto {
        fs::create_dir_all(&proto.output).with_context(|| {
            format!(
                "Failed to create directory at path {:?} for proto output '{}'",
                proto.output,
                proto.lang.as_config()
            )
        })?;
    }
    Ok(())
}

fn arg_with_value(arg: &str, value: &str) -> String {
    ["--", arg, "=", value].concat()
}

fn quote(val: &str) -> String {
    ["\"", val, "\""].concat()
}

fn normalize_slashes(path: &str) -> String {
    path.replace("\\", "/")
}

fn protoc_path() -> PathBuf {
    prost_build::protoc()
}

#[cfg(test)]
mod tests {
    use crate::run::{arg_with_value, quote};
    use crate::Options;
    use std::path::Path;

    mod collect_and_validate_args {
        use crate::lang::Lang;
        use crate::lang_option::LangOption;
        use crate::run::tests::assert_arg_pair_exists;
        use crate::run::{
            collect_and_validate_args, collect_proto_outputs, collect_proto_path,
            create_output_paths, PROTOC_ARG_PROTO_PATH,
        };
        use crate::Options;
        use anyhow::Result;
        use std::path::PathBuf;
        use std::{env, fs};
        use tempfile::tempdir;

        #[test]
        fn proto_path() -> Result<()> {
            let input = env::current_dir().unwrap().to_str().unwrap().to_string();
            let mut options = Options::default();
            options.input = PathBuf::from(&input);
            let args = collect_and_validate_args(&options)?;
            assert_arg_pair_exists(&args, &PROTOC_ARG_PROTO_PATH, &input);
            Ok(())
        }

        #[test]
        fn proto_path_missing() {
            let input = "definitely/missing/path";
            let mut options = Options::default();
            options.input = PathBuf::from(input);
            let mut args = Vec::new();
            assert!(collect_proto_path(&options, &mut args).is_err());
        }

        #[test]
        fn proto_output() -> Result<()> {
            let mut options = Options::default();
            let cpp = LangOption {
                lang: Lang::Cpp,
                output: PathBuf::from("cpp/path"),
                output_prefix: PathBuf::new(),
            };
            let csharp = LangOption {
                lang: Lang::CSharp,
                output: PathBuf::from("csharp/path"),
                output_prefix: PathBuf::new(),
            };
            options.proto.push(cpp);
            options.proto.push(csharp);
            let mut args = Vec::new();
            collect_proto_outputs(&options, &mut args)?;
            assert_arg_pair_exists(&args, "cpp_out", "cpp/path");
            assert_arg_pair_exists(&args, "csharp_out", "csharp/path");
            Ok(())
        }

        #[test]
        fn creates_all_proto_output_dirs() -> Result<()> {
            let tempdir = tempdir()?;
            let mut options = Options::default();
            let cpp_path = tempdir.path().join("cpp");
            let csharp_path = tempdir.path().join("csharp");
            options.proto.push(LangOption {
                lang: Lang::Cpp,
                output: cpp_path.clone(),
                output_prefix: PathBuf::new(),
            });
            options.proto.push(LangOption {
                lang: Lang::Cpp,
                output: csharp_path.clone(),
                output_prefix: PathBuf::new(),
            });
            create_output_paths(&options)?;
            assert!(fs::read_dir(&cpp_path).is_ok());
            assert!(fs::read_dir(&csharp_path).is_ok());
            Ok(())
        }
    }

    mod collect_input_files {
        use crate::run::collect_input_files;
        use crate::run::tests::options_with_input;
        use anyhow::Result;
        use std::fs;
        use std::path::{Path, PathBuf};
        use tempfile::tempdir;

        #[test]
        fn collects_all_in_dir() -> Result<()> {
            let dir = tempdir()?;
            let root = dir.path();
            create_files_at(root, &["aaa.proto", "bbb.proto", "ccc.proto"])?;
            let files = collect_input_files(&options_with_input(root))?;
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
            let files = collect_input_files(&options_with_input(root))?;
            assert_eq!(files.len(), 6);
            Ok(())
        }

        #[test]
        fn paths_are_absolute() -> Result<()> {
            let dir = tempdir()?;
            let root = dir.path();

            create_files_at(root, &["aaa.proto"])?;
            let files = collect_input_files(&options_with_input(root))?;
            assert_arg_equal_to_path(files.get(0).unwrap(), "aaa.proto");

            create_files_at(root, &["a/b/c/aaa.proto"])?;
            let files = collect_input_files(&options_with_input(root))?;
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
            let files = collect_input_files(&options_with_input(root))?;
            assert_arg_equal_to_path(files.get(0).unwrap(), "aaa.proto");
            Ok(())
        }

        fn assert_arg_equal_to_path(arg: &str, path: &str) {
            assert_eq!(PathBuf::from(arg).as_path(), PathBuf::from(path));
        }

        fn unquote(val: &str) -> &str {
            &val[1..val.len() - 1]
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
        use crate::run::is_proto_ext;
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

    fn options_with_input(path: &Path) -> Options {
        let mut options = Options::default();
        options.input = path.to_path_buf();
        options
    }

    /// Checks for --arg=value and --arg value. Asserts if neither are found.
    fn assert_arg_pair_exists(args: &Vec<String>, first: &str, second: &str) {
        if args.contains(&arg_with_value(first, second)) {
            return;
        }
        let pos = args
            .iter()
            .position(|arg| *arg == ["--", first].concat())
            .expect(&format!("expected arg not found in list: --{}", first));
        assert!(
            pos < args.len(),
            "no more elements found after first arg: --{}",
            first
        );
        assert_eq!(
            args.get(pos + 1)
                .expect(&format!("missing value for arg: --{}", first)),
            &quote(second)
        );
    }
}
