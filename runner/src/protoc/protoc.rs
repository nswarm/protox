use crate::{util, Config};
use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
        let mut args = vec![collect_proto_path(config)?];
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

        info!("using protoc at path: {:?}", protoc_path);
        info!("running command:\tprotoc {}", self.args.join(" "));

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

fn collect_proto_path(config: &Config) -> Result<String> {
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
    Ok(arg_with_value(PROTOC_ARG_PROTO_PATH, input))
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
        arg_with_value, collect_extra_protoc_args, collect_proto_path, PROTOC_ARG_PROTO_PATH,
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
        let arg = collect_proto_path(&config)?;
        assert_eq!(arg, arg_with_value(PROTOC_ARG_PROTO_PATH, &input));
        Ok(())
    }

    #[test]
    fn proto_path_missing() {
        let input = "definitely/missing/path";
        let mut config = Config::default();
        config.input = PathBuf::from(input);
        assert!(collect_proto_path(&config).is_err());
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

    pub fn quote_arg(arg: &str) -> String {
        ["\"", arg, "\""].concat()
    }
}
