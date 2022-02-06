use crate::{util, Config};
use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use util::DisplayNormalized;

const PROTOC_ARG_PROTO_PATH: &str = "proto_path";
const PROTOC_ARG_DESCRIPTOR_SET_OUT: &str = "descriptor_set_out";
const PROTOC_ARG_INCLUDE_SOURCE_INFO: &str = "include_source_info";

/// Manages collecting args and the invocation of `protoc`, the protobuf compiler.
pub struct Protoc {
    args: Vec<String>,
    input_files: Vec<String>,
}

impl Protoc {
    pub fn new(config: &Config) -> Result<Protoc> {
        let mut args = collect_proto_paths(config)?;
        let descriptor_set_path = config
            .descriptor_set_path
            .to_str()
            .ok_or(anyhow!("Descriptor set path is not valid unicode."))?;
        // Descriptor set with source info is used by generators.
        args.push(arg_with_value(
            PROTOC_ARG_DESCRIPTOR_SET_OUT,
            descriptor_set_path,
        ));
        args.push(["--", PROTOC_ARG_INCLUDE_SOURCE_INFO].concat());
        args.append(&mut collect_extra_protoc_args(config));
        Ok(Self {
            args,
            input_files: Vec::new(),
        })
    }

    pub fn execute(&mut self) -> Result<()> {
        let protoc_path = protoc_path();
        self.args.append(&mut self.input_files.clone());

        info!("using protoc at path: {}", protoc_path.display_normalized());
        info!(
            "running command:\tprotoc {}",
            util::normalize_slashes(self.args.join(" "))
        );

        let mut child = Command::new(&protoc_path)
            .args(&self.args)
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
            Err(anyhow!("protoc exited with status {}", status))
        }
    }

    pub fn add_args(&mut self, args: &mut Vec<String>) {
        self.args.append(args);
    }

    pub fn add_input_files(&mut self, input_files: &mut Vec<String>) {
        // Cache input files until execute since they must come last in protoc args.
        self.input_files.append(input_files);
    }
}

fn collect_proto_paths(config: &Config) -> Result<Vec<String>> {
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
    let mut args = vec![arg_with_value(PROTOC_ARG_PROTO_PATH, input)];
    for include in &config.includes {
        args.push(arg_with_value(PROTOC_ARG_PROTO_PATH, include));
    }
    Ok(args)
}

fn collect_extra_protoc_args(config: &Config) -> Vec<String> {
    config
        .extra_protoc_args
        .iter()
        .map(|s| util::unquote_arg(s))
        .collect()
}

fn protoc_path() -> PathBuf {
    match option_env!("PROTOC_EXE") {
        None => PathBuf::from("protoc"),
        Some(path) => PathBuf::from(path),
    }
}

pub fn arg_with_value(arg: &str, value: &str) -> String {
    ["--", arg, "=", value].concat()
}

#[cfg(test)]
mod tests {
    use crate::protoc::protoc::{
        arg_with_value, collect_extra_protoc_args, collect_proto_paths, PROTOC_ARG_PROTO_PATH,
    };
    use crate::Config;
    use anyhow::Result;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn proto_path() -> Result<()> {
        let input = env::current_dir().unwrap().to_str().unwrap().to_string();
        let mut config = Config::default();
        config.input = PathBuf::from(&input);
        let proto_paths = collect_proto_paths(&config)?;
        assert_eq!(
            proto_paths,
            vec![arg_with_value(PROTOC_ARG_PROTO_PATH, &input)]
        );
        Ok(())
    }

    #[test]
    fn proto_path_missing() {
        let input = "definitely/missing/path";
        let mut config = Config::default();
        config.input = PathBuf::from(input);
        assert!(collect_proto_paths(&config).is_err());
    }

    #[test]
    fn passes_extra_protoc_args() -> Result<()> {
        let mut config = Config::default();
        let extra_protoc_args = vec!["--test1", "--test2=hello"];
        for extra_arg in &extra_protoc_args {
            config.extra_protoc_args.push(quote_arg(extra_arg));
        }
        let args = collect_extra_protoc_args(&config);
        assert_eq!(extra_protoc_args, args);
        Ok(())
    }

    #[test]
    fn collects_extra_includes() -> Result<()> {
        let mut config = Config::default();
        config.includes = vec!["include0".to_string(), "include1".to_string()];
        let args = collect_proto_paths(&config)?;
        for include in config.includes {
            assert!(args.contains(&arg_with_value(PROTOC_ARG_PROTO_PATH, &include)));
        }
        Ok(())
    }

    fn quote_arg(arg: &str) -> String {
        ["\"", arg, "\""].concat()
    }
}
