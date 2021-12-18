use crate::Config;
use anyhow::{anyhow, Context, Result};
use log::info;
use std::path::PathBuf;
use std::process::Command;

/// Manages collecting args and the invocation of `protoc`, the protobuf compiler.
pub struct Protoc<'a> {
    config: &'a Config,
    pub args: Vec<String>,
}

impl<'a> Protoc<'a> {
    pub fn new(config: &'a Config) -> Protoc<'a> {
        Self {
            config,
            args: vec![],
        }
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

fn protoc_path() -> PathBuf {
    match option_env!("PROTOC_EXE") {
        None => PathBuf::from("protoc"),
        Some(path) => PathBuf::from(path),
    }
}
