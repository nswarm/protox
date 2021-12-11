use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
        let args = self.collect_and_validate_args()?;

        info!("using protoc at path: {:?}", protoc_path);
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

    fn collect_and_validate_args(&self) -> Result<Vec<String>> {
        let mut args = Vec::new();
        self.collect_proto_path(&mut args)?;
        Ok(args)
    }

    fn collect_proto_path(&self, args: &mut Vec<String>) -> Result<()> {
        if let Err(_) = fs::read_dir(&self.options.input) {
            bail!(
                "Invalid input: could not find the directory located at path '{:?}'.",
                self.options.input
            );
        }
        let input = match self.options.input.to_str() {
            None => bail!("Invalid input: Could not parse path to string."),
            Some(input) => input,
        };
        args.push(PROTOC_ARG_PROTO_PATH.to_string());
        args.push(input.to_string());
        Ok(())
    }
}

fn protoc_path() -> PathBuf {
    prost_build::protoc()
}

#[cfg(test)]
mod tests {
    mod collect_and_validate_args {
        use crate::runner::tests::assert_arg_pair_exists;
        use crate::runner::PROTOC_ARG_PROTO_PATH;
        use crate::{Options, Protoc};
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn proto_path() -> Result<()> {
            let input = env::current_dir().unwrap().to_str().unwrap().to_string();
            let mut options = Options::default();
            options.input = PathBuf::from(&input);
            let protoc = Protoc::with_options(options);
            let args = protoc.collect_and_validate_args()?;
            assert!(args.contains(&PROTOC_ARG_PROTO_PATH.to_string()));
            assert_arg_pair_exists(&args, &PROTOC_ARG_PROTO_PATH, &input);
            Ok(())
        }

        #[test]
        fn proto_path_missing() {
            let input = "definitely/missing/path";
            let mut options = Options::default();
            options.input = PathBuf::from(input);
            let protoc = Protoc::with_options(options);
            let mut args = Vec::new();
            assert!(protoc.collect_proto_path(&mut args).is_err());
        }
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
