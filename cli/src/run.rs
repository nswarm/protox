use crate::lang::Lang;
use crate::Options;
use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PROTOC_ARG_PROTO_PATH: &str = "proto_path";

pub fn protoc(options: &Options) -> Result<()> {
    let protoc_path = protoc_path();
    let args = collect_and_validate_args(options)?;

    info!("using protoc at path: {:?}", protoc_path);
    info!("running:\nprotoc {:?}", args.join(" "));

    // todo create_output_path

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
    collect_proto_path(options, &mut args)?;
    collect_proto_outputs(options, &mut args)?;
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

fn create_output_path(output_path: &PathBuf, lang: &Lang) -> Result<()> {
    fs::create_dir_all(output_path).with_context(|| {
        format!(
            "Failed to create directory at path {:?} for proto output '{}'",
            output_path,
            lang.as_config()
        )
    })?;
    Ok(())
}

fn arg_with_value(arg: &str, value: &str) -> String {
    ["--", arg, "=", value].concat()
}

fn protoc_path() -> PathBuf {
    prost_build::protoc()
}

#[cfg(test)]
mod tests {
    mod collect_and_validate_args {
        use crate::lang::Lang;
        use crate::lang_option::LangOption;
        use crate::run::tests::assert_arg_pair_exists;
        use crate::run::{collect_and_validate_args, collect_proto_path, PROTOC_ARG_PROTO_PATH};
        use crate::Options;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

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
            };
            let csharp = LangOption {
                lang: Lang::CSharp,
                output: PathBuf::from("csharp/path"),
            };
            options.proto.push(cpp);
            options.proto.push(csharp);
            let args = collect_and_validate_args(&options)?;
            assert_arg_pair_exists(&args, "cpp_out", "cpp/path");
            assert_arg_pair_exists(&args, "csharp_out", "csharp/path");
            Ok(())
        }
    }

    /// Checks for --arg=value and --arg value. Asserts if neither are found.
    fn assert_arg_pair_exists(args: &Vec<String>, first: &str, second: &str) {
        if args.contains(&["--", first, "=", second].concat()) {
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
