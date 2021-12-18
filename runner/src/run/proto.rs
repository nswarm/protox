use crate::lang::Lang;
use crate::Config;
use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROTOC_ARG_PROTO_PATH: &str = "proto_path";

pub const SUPPORTED_LANGUAGES: [Lang; 9] = [
    Lang::Cpp,
    Lang::CSharp,
    Lang::Java,
    Lang::Javascript,
    Lang::Kotlin,
    Lang::ObjectiveC,
    Lang::Php,
    Lang::Python,
    Lang::Ruby,
];

pub fn run(config: &Config, input_files: &Vec<String>) -> Result<()> {
    basic(config, input_files)?;
    rust(config, input_files)?;
    Ok(())
}

/// Any basic protoc support.
fn basic(config: &Config, input_files: &Vec<String>) -> Result<()> {
    if !has_any_supported_language(config) {
        return Ok(());
    }

    let protoc_path = protoc_path();
    let args = collect_and_validate_args(config, input_files)?;

    info!("using protoc at path: {:?}", protoc_path);
    info!("running command:\tprotoc {}", args.join(" "));

    create_output_dirs(config)?;

    let mut child = Command::new(&protoc_path)
        .args(args)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to spawn protoc process using protoc: {:?}",
                protoc_path
            )
        })?;
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Exited with status {}", status))
    }
}

/// Special case since rust uses prost plugin.
fn rust(config: &Config, input_files: &Vec<String>) -> Result<()> {
    let rust_config = config
        .proto
        .iter()
        .find(|lang_config| lang_config.lang == Lang::Rust);
    let output = match rust_config {
        None => return Ok(()),
        Some(rust_config) => &rust_config.output,
    };

    create_output_dir(output)?;

    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir(output);
    for extra_arg in &config.extra_protoc_args {
        prost_config.protoc_arg(unquote_arg(extra_arg));
    }
    prost_config.compile_protos(input_files, &[&config.input])?;
    Ok(())
}

fn create_output_dir(output: &Path) -> Result<()> {
    fs::create_dir_all(&output).with_context(|| {
        format!(
            "Failed to create directory at path {:?} for proto output '{}'",
            output,
            Lang::Rust.as_config(),
        )
    })?;
    Ok(())
}

fn collect_and_validate_args(config: &Config, input_files: &Vec<String>) -> Result<Vec<String>> {
    let mut args = Vec::new();
    collect_proto_path(config, &mut args).context("Failed to collect proto_path arg.")?;
    collect_proto_outputs(config, &mut args).context("Failed to collect proto output args.")?;
    collect_extra_protoc_args(config, &mut args);
    // Input files must always come last.
    args.append(&mut input_files.clone());
    Ok(args)
}

fn collect_proto_path(config: &Config, args: &mut Vec<String>) -> Result<()> {
    if let Err(_) = fs::read_dir(&config.input) {
        bail!(
            "Invalid input: could not find the directory located at path '{:?}'.",
            config.input
        );
    }
    let input = match config.input.to_str() {
        None => bail!("Invalid input: Could not parse path to string."),
        Some(input) => input,
    };
    args.push(arg_with_value(PROTOC_ARG_PROTO_PATH, input));
    Ok(())
}

fn collect_proto_outputs(config: &Config, args: &mut Vec<String>) -> Result<()> {
    for proto in &config.proto {
        if !SUPPORTED_LANGUAGES.contains(&proto.lang) {
            continue;
        }
        let arg = [proto.lang.as_config().as_str(), "_out"].concat();
        let value = proto
            .output
            .to_str()
            .ok_or(anyhow!("Output path is invalid: {:?}", proto.output))?;
        args.push(arg_with_value(&arg, value));
    }
    Ok(())
}

fn collect_extra_protoc_args(config: &Config, args: &mut Vec<String>) {
    for arg in &config.extra_protoc_args {
        args.push(unquote_arg(arg));
    }
}

fn create_output_dirs(config: &Config) -> Result<()> {
    for proto in &config.proto {
        if !SUPPORTED_LANGUAGES.contains(&proto.lang) {
            continue;
        }
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

fn has_any_supported_language(config: &Config) -> bool {
    let count = config
        .proto
        .iter()
        .filter(|c| SUPPORTED_LANGUAGES.contains(&c.lang))
        .count();
    count > 0
}

fn arg_with_value(arg: &str, value: &str) -> String {
    ["--", arg, "=", value].concat()
}

fn protoc_path() -> PathBuf {
    match option_env!("PROTOC_EXE") {
        None => PathBuf::from("protoc"),
        Some(path) => PathBuf::from(path),
    }
}

pub fn unquote_arg(arg: &str) -> String {
    arg[1..arg.len() - 1].to_string()
}

#[cfg(test)]
mod tests {
    use crate::lang::Lang;
    use crate::lang_config::LangConfig;
    use crate::run::proto::{arg_with_value, has_any_supported_language};
    use crate::run::proto::{
        collect_and_validate_args, collect_extra_protoc_args, collect_proto_outputs,
        collect_proto_path, create_output_dirs, PROTOC_ARG_PROTO_PATH,
    };
    use crate::Config;
    use anyhow::Result;
    use std::path::PathBuf;
    use std::{env, fs};
    use tempfile::tempdir;

    #[test]
    fn proto_path() -> Result<()> {
        let input = env::current_dir().unwrap().to_str().unwrap().to_string();
        let mut config = Config::default();
        config.input = PathBuf::from(&input);
        let args = collect_and_validate_args(&config, &Vec::new())?;
        assert_arg_pair_exists(&args, &PROTOC_ARG_PROTO_PATH, &input);
        Ok(())
    }

    #[test]
    fn proto_path_missing() {
        let input = "definitely/missing/path";
        let mut config = Config::default();
        config.input = PathBuf::from(input);
        let mut args = Vec::new();
        assert!(collect_proto_path(&config, &mut args).is_err());
    }

    #[test]
    fn proto_output() -> Result<()> {
        let mut config = Config::default();
        let cpp = LangConfig {
            lang: Lang::Cpp,
            output: PathBuf::from("cpp/path"),
            output_prefix: PathBuf::new(),
        };
        let csharp = LangConfig {
            lang: Lang::CSharp,
            output: PathBuf::from("csharp/path"),
            output_prefix: PathBuf::new(),
        };
        config.proto.push(cpp);
        config.proto.push(csharp);
        let mut args = Vec::new();
        collect_proto_outputs(&config, &mut args)?;
        assert_arg_pair_exists(&args, "cpp_out", "cpp/path");
        assert_arg_pair_exists(&args, "csharp_out", "csharp/path");
        Ok(())
    }

    #[test]
    fn ignores_unsupported_languages() -> Result<()> {
        let mut config = Config::default();
        let rust = LangConfig {
            lang: Lang::Rust,
            output: PathBuf::from("rust/path"),
            output_prefix: PathBuf::new(),
        };
        config.proto.push(rust);
        let mut args = Vec::new();
        collect_proto_outputs(&config, &mut args)?;
        assert_eq!(args.len(), 0);
        Ok(())
    }

    #[test]
    fn creates_all_proto_output_dirs() -> Result<()> {
        let tempdir = tempdir()?;
        let mut config = Config::default();
        let cpp_path = tempdir.path().join("cpp");
        let csharp_path = tempdir.path().join("csharp");
        config.proto.push(LangConfig {
            lang: Lang::Cpp,
            output: cpp_path.clone(),
            output_prefix: PathBuf::new(),
        });
        config.proto.push(LangConfig {
            lang: Lang::Cpp,
            output: csharp_path.clone(),
            output_prefix: PathBuf::new(),
        });
        create_output_dirs(&config)?;
        assert!(fs::read_dir(&cpp_path).is_ok());
        assert!(fs::read_dir(&csharp_path).is_ok());
        Ok(())
    }

    #[test]
    fn passes_extra_protoc_args() -> Result<()> {
        let mut config = Config::default();
        let extra_protoc_args = vec!["--test1", "--test2=hello"];
        for extra_arg in &extra_protoc_args {
            config.extra_protoc_args.push(quote_arg(extra_arg));
        }
        let mut out_args = vec![];
        collect_extra_protoc_args(&config, &mut out_args);
        assert_eq!(extra_protoc_args, out_args);
        Ok(())
    }

    pub fn quote_arg(arg: &str) -> String {
        ["\"", arg, "\""].concat()
    }

    #[test]
    fn has_supported_language() {
        let mut config = Config::default();
        config.proto.push(LangConfig {
            lang: Lang::Cpp,
            output: Default::default(),
            output_prefix: Default::default(),
        });
        assert!(has_any_supported_language(&config));
    }

    #[test]
    fn has_no_supported_language() {
        let mut config = Config::default();
        config.proto.push(LangConfig {
            lang: Lang::Rust,
            output: Default::default(),
            output_prefix: Default::default(),
        });
        assert!(!has_any_supported_language(&config));
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
            second
        );
    }
}
