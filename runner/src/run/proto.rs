use crate::lang::Lang;
use crate::run::protoc::{arg_with_value, Protoc};
use crate::run::util;
use crate::Config;
use anyhow::{anyhow, Context, Result};

const BASIC_SUPPORTED_LANGUAGES: [Lang; 9] = [
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

pub fn supported_languages() -> Vec<Lang> {
    let mut vec = BASIC_SUPPORTED_LANGUAGES.to_vec();
    vec.push(Lang::Rust);
    vec
}

pub fn run(config: &Config, protoc: &mut Protoc) -> Result<()> {
    util::check_languages_supported("proto", &config.proto, &supported_languages())?;
    util::create_output_dirs(&config.proto)?;
    run_builtin(config, protoc)?;
    run_rust(config, protoc)?;
    Ok(())
}

/// Any basic protoc support.
fn run_builtin(config: &Config, protoc: &mut Protoc) -> Result<()> {
    if !has_any_supported_language(config) {
        return Ok(());
    }
    protoc.add_args(
        &mut collect_proto_outputs(config).context("Failed to collect proto output args.")?,
    );
    protoc.flag_for_execution();
    Ok(())
}

/// Special case since rust uses prost plugin.
fn run_rust(config: &Config, protoc: &Protoc) -> Result<()> {
    let rust_config = match config
        .proto
        .iter()
        .find(|lang_config| lang_config.lang == Lang::Rust)
    {
        None => return Ok(()),
        Some(config) => config,
    };

    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir(&rust_config.output);
    for extra_arg in &config.extra_protoc_args {
        prost_config.protoc_arg(unquote_arg(extra_arg));
    }
    prost_config.compile_protos(protoc.input_files(), &[&config.input])?;
    Ok(())
}

pub fn unquote_arg(arg: &str) -> String {
    arg[1..arg.len() - 1].to_string()
}

fn collect_proto_outputs(config: &Config) -> Result<Vec<String>> {
    let mut args = Vec::new();
    for proto in &config.proto {
        if !BASIC_SUPPORTED_LANGUAGES.contains(&proto.lang) {
            continue;
        }
        let arg = [proto.lang.as_config().as_str(), "_out"].concat();
        let value = proto
            .output
            .to_str()
            .ok_or(anyhow!("Output path is invalid: {:?}", proto.output))?;
        args.push(arg_with_value(&arg, value));
    }
    Ok(args)
}

fn has_any_supported_language(config: &Config) -> bool {
    let count = config
        .proto
        .iter()
        .filter(|c| BASIC_SUPPORTED_LANGUAGES.contains(&c.lang))
        .count();
    count > 0
}

#[cfg(test)]
mod tests {
    use crate::lang::Lang;
    use crate::lang_config::LangConfig;
    use crate::run::proto::collect_proto_outputs;
    use crate::run::proto::has_any_supported_language;
    use crate::run::protoc::arg_with_value;
    use crate::Config;
    use anyhow::Result;
    use std::path::PathBuf;

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
        let args = collect_proto_outputs(&config)?;
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
        let args = collect_proto_outputs(&config)?;
        assert_eq!(args.len(), 0);
        Ok(())
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
