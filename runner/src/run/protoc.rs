use crate::Config;
use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PROTOC_ARG_PROTO_PATH: &str = "proto_path";

/// Manages collecting args and the invocation of `protoc`, the protobuf compiler.
pub struct Protoc<'a> {
    config: &'a Config,
    pub args: Vec<String>,
}

impl<'a> Protoc<'a> {
    pub fn new(config: &'a Config) -> Result<Protoc<'a>> {
        let args = vec![collect_proto_path(config)?];
        Ok(Self { config, args })
    }

    pub fn execute(&self) -> Result<()> {
        let protoc_path = protoc_path();

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
            Err(anyhow!("Exited with status {}", status))
        }
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
    use crate::run::protoc::{arg_with_value, collect_proto_path, PROTOC_ARG_PROTO_PATH};
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
}
