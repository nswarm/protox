use crate::lang::Lang;
use crate::lang_config::LangConfig;
use crate::protoc::protoc::{arg_with_value, Protoc};
use crate::{util, Config};
use anyhow::{anyhow, Context, Result};

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

pub fn register(config: &Config, protoc: &mut Protoc) -> Result<()> {
    util::create_output_dirs(
        &config
            .protos
            .iter()
            .filter(|cfg| SUPPORTED_LANGUAGES.contains(&cfg.lang))
            .collect::<Vec<&LangConfig>>(),
    )?;
    register_builtin(config, protoc)?;
    Ok(())
}

/// Any basic protoc support.
fn register_builtin(config: &Config, protoc: &mut Protoc) -> Result<()> {
    if !has_any_supported_language(config) {
        return Ok(());
    }
    protoc.add_args(
        &mut collect_proto_outputs(config).context("Failed to collect proto output args.")?,
    );
    Ok(())
}

fn collect_proto_outputs(config: &Config) -> Result<Vec<String>> {
    let mut args = Vec::new();
    for proto in &config.protos {
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
    Ok(args)
}

fn has_any_supported_language(config: &Config) -> bool {
    let count = config
        .protos
        .iter()
        .filter(|c| SUPPORTED_LANGUAGES.contains(&c.lang))
        .count();
    count > 0
}

#[cfg(test)]
mod tests {
    use crate::lang::Lang;
    use crate::lang_config::LangConfig;
    use crate::protoc::builtin::collect_proto_outputs;
    use crate::protoc::builtin::has_any_supported_language;
    use crate::protoc::protoc::arg_with_value;
    use crate::Config;
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn proto_output() -> Result<()> {
        let mut config = Config::default();
        let cpp = LangConfig {
            lang: Lang::Cpp,
            output: PathBuf::from("cpp/path"),
        };
        let csharp = LangConfig {
            lang: Lang::CSharp,
            output: PathBuf::from("csharp/path"),
        };
        config.protos.push(cpp);
        config.protos.push(csharp);
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
        };
        config.protos.push(rust);
        let args = collect_proto_outputs(&config)?;
        assert_eq!(args.len(), 0);
        Ok(())
    }

    #[test]
    fn has_supported_language() {
        let mut config = Config::default();
        config.protos.push(LangConfig {
            lang: Lang::Cpp,
            output: Default::default(),
        });
        assert!(has_any_supported_language(&config));
    }

    #[test]
    fn has_no_supported_language() {
        let mut config = Config::default();
        config.protos.push(LangConfig {
            lang: Lang::Rust,
            output: Default::default(),
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
