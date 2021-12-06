use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use log::info;

use crate::Options;

const PROTOC_ARG_PROTO_PATH: &str = "--proto_path";

pub struct Protoc {
    options: Options,
}

impl Protoc {
    pub fn with_options(options: Options) -> Self {
        Self { options }
    }

    pub fn run(self) -> Result<()> {
        let protoc_path = protoc_path();
        info!("using protoc at path: {:?}", protoc_path);
        let args = self.collect_args();
        info!("running:\nprotoc {:?}", args.join(" "));

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

    fn collect_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        self.push_proto_path(&mut args);
        args
    }

    fn push_proto_path(&self, args: &mut Vec<String>) {
        args.push(PROTOC_ARG_PROTO_PATH.to_string());
        args.push(self.options.input.to_str().unwrap().to_string());
    }
}

fn protoc_path() -> PathBuf {
    prost_build::protoc()
}

#[cfg(test)]
mod tests {
    use crate::runner::PROTOC_ARG_PROTO_PATH;
    use crate::{Options, Protoc};
    use std::path::PathBuf;

    #[test]
    fn proto_path() {
        let input = "test/input";
        let mut options = Options::default();
        options.input = PathBuf::from(input);
        let protoc = Protoc::with_options(options);
        let args = protoc.collect_args();
        assert!(args.contains(&PROTOC_ARG_PROTO_PATH.to_string()));
        assert_arg_pair_exists(&args, &PROTOC_ARG_PROTO_PATH, &input);
    }

    fn assert_arg_pair_exists(args: &Vec<String>, first: &str, second: &str) {
        let pos = args
            .iter()
            .position(|arg| arg == first)
            .expect(format!("{} should exist in args", first).as_str());
        assert!(
            pos < args.len(),
            "no more elements found after first arg in pair"
        );
        assert_eq!(args.get(pos + 1).expect("missing second arg"), second);
    }
}
