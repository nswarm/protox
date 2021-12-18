use crate::idl::Idl;
use crate::lang::Lang;
use crate::lang_config::LangConfig;
use anyhow::{anyhow, Error, Result};
use clap::{crate_version, App, Arg, ArgMatches, Values};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub const APP_NAME: &str = "protoffi";
pub const IDL: &str = "idl";
pub const INPUT: &str = "input";
pub const OUTPUT_ROOT: &str = "output-root";
pub const PROTO: &str = "proto";
pub const SERVER: &str = "server";
pub const CLIENT: &str = "client";
pub const DIRECT: &str = "direct";
pub const OUTPUT_TYPES: [&str; 4] = [PROTO, SERVER, CLIENT, DIRECT];
pub const OUTPUT_VALUE_NAME: &str = "LANG[=OUTPUT]";
pub const OUTPUT_SEPARATOR: &str = "=";
pub const PLUGIN_PROTO: &str = "plugin-proto";
pub const PROTOC_ARGS: &str = "protoc-args";
pub const OUTPUT_LONG_ABOUT: & str = "If OUTPUT is a relative path, it is evaluated relative to OUTPUT_ROOT if set, or the current working directory otherwise.";
pub const LONG_ABOUT_NEWLINE: &str = "\n\n";

const DISPLAY_LAST: usize = 990;

const PROTO_SUPPORTED_LANGUAGES: [Lang; 10] = [
    Lang::Cpp,
    Lang::CSharp,
    Lang::Java,
    Lang::Javascript,
    Lang::Kotlin,
    Lang::ObjectiveC,
    Lang::Php,
    Lang::Python,
    Lang::Ruby,
    Lang::Rust,
];

fn parse_cli_args<I, T>(iter: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    App::new(APP_NAME)
        .long_about("protoffi is an executable that generates C-ABI-compatible code in one or more languages for seamless and performant direct usage of those types across the library boundary.")
        .version(crate_version!())
        .args([
            Arg::new(IDL)
                .display_order(0)
                .about("IDL type of files expected at the INPUT path.")
                .long(IDL)
                .default_value(&Idl::Proto.as_config()),

            Arg::new(INPUT)
                .display_order(1)
                .about("File path to search for IDL files.")
                .default_short()
                .long(INPUT)
                .takes_value(true)
                .required(true),

            Arg::new(OUTPUT_ROOT)
                .display_order(2)
                .about("All output files will be prefixed with this path.")
                .short('r')
                .long(OUTPUT_ROOT)
                .takes_value(true),

            output_arg(PROTO)
                .display_order(100)
                .long_about(&join_about(&[
                    "Indicates protobuf code should be generated for language LANG to file path OUTPUT.",
                    "If OUTPUT is not provided, it defaults to `proto_<LANG>`.",
                    OUTPUT_LONG_ABOUT,
                    &format!("Supported languages for LANG: {}. \
                    Custom support can be added via the used of {}.", supported_languages(), PLUGIN_PROTO),
                ])),

            output_arg(SERVER)
                .display_order(101),

            output_arg(CLIENT)
                .display_order(102),

            output_arg(DIRECT)
                .display_order(103),

            Arg::new(PROTOC_ARGS)
                .display_order(DISPLAY_LAST)
                .long_about(&format!("Add any arguments directly to protoc invocation. Note they must be wrapped with \"\" as to not be picked up as arguments to protoffi.\nFor example: --{} \"--descriptor_set_out=FILE\"", PROTOC_ARGS))
                .requires(PROTO)
                .long(PROTOC_ARGS)
                .takes_value(true)
                .multiple_values(true),

        ]).get_matches_from(iter)
}

fn output_arg(name: &str) -> Arg {
    Arg::new(name)
        .default_short()
        .long(name)
        .required_unless_present_any(&OUTPUT_TYPES)
        .value_name(&OUTPUT_VALUE_NAME)
        .takes_value(true)
        .multiple_values(true)
}

fn join_about(lines: &[&str]) -> String {
    lines.join(LONG_ABOUT_NEWLINE).to_string()
}

#[derive(Default)]
pub struct Config {
    pub idl: Idl,
    pub input: PathBuf,
    pub output_root: Option<PathBuf>,
    pub proto: Vec<LangConfig>,
    pub extra_protoc_args: Vec<String>,
}

impl Config {
    pub fn from_cli() -> Result<Self> {
        let args = parse_cli_args(&mut env::args_os());
        let config = Config::from_args(&args)?;
        Ok(config)
    }

    pub fn from_args(args: &ArgMatches) -> Result<Self> {
        let output_root = parse_output_root(&args);
        Ok(Self {
            idl: Idl::from_args(&args)?,
            input: parse_input(&args)?,
            output_root: output_root.clone(),
            proto: parse_proto_outputs(&args, output_root.as_ref())?,
            extra_protoc_args: parse_extra_protoc_args(&args),
        })
    }
}

fn parse_input(args: &ArgMatches) -> Result<PathBuf> {
    match args.value_of(INPUT) {
        None => Err(error_missing_required_arg(INPUT)),
        Some(input) => Ok(PathBuf::from(input)),
    }
}

fn parse_output_root(args: &ArgMatches) -> Option<PathBuf> {
    args.value_of(OUTPUT_ROOT)
        .and_then(|value| Some(PathBuf::from(value)))
}

fn parse_proto_outputs(
    args: &ArgMatches,
    output_root: Option<&PathBuf>,
) -> Result<Vec<LangConfig>> {
    let mut proto_outputs = Vec::new();
    let values = match args.values_of(PROTO) {
        None => return Ok(proto_outputs),
        Some(values) => values,
    };
    for value in values {
        proto_outputs.push(LangConfig::from_config(value, output_root, PROTO)?);
    }
    Ok(proto_outputs)
}

fn parse_extra_protoc_args(args: &ArgMatches) -> Vec<String> {
    args.values_of(PROTOC_ARGS)
        .unwrap_or(Values::default())
        .map(&str::to_string)
        .collect()
}

fn error_missing_required_arg(name: &str) -> Error {
    anyhow!("Missing required argument '--{}'", name)
}

fn supported_languages() -> String {
    PROTO_SUPPORTED_LANGUAGES
        .map(|lang| lang.as_config())
        .join(", ")
}

trait ArgExt {
    fn default_short(self) -> Self;
}

impl ArgExt for Arg<'_> {
    fn default_short(self) -> Self {
        let short = self.get_name().chars().nth(0).unwrap();
        self.short(short)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{parse_cli_args, APP_NAME, INPUT, OUTPUT_ROOT, PROTO, PROTOC_ARGS};
    use crate::Config;
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn parse_input() -> Result<()> {
        let input = "path/to/input";
        let config = Config::from_args(&parse_cli_args([
            APP_NAME,
            &arg(INPUT),
            input,
            &arg(PROTO),
            "cpp",
        ]))?;
        assert_eq!(config.input, PathBuf::from(input));
        Ok(())
    }

    #[test]
    fn parse_output_root() -> Result<()> {
        let output_root = "path/to/output";
        let config = config_with_required_args([&arg(OUTPUT_ROOT), output_root])?;
        assert_eq!(config.output_root, Some(PathBuf::from(output_root)));
        Ok(())
    }

    #[test]
    fn parse_extra_protoc_args() -> Result<()> {
        let extra_args = [
            quote("--custom-arg=i/am/a/path"),
            quote("--other-arg=\"string\""),
        ];
        let protoc_args = arg(PROTOC_ARGS);
        let config = config_with_required_args([&protoc_args, &extra_args[0], &extra_args[1]])?;
        assert_eq!(config.extra_protoc_args.get(0), Some(&extra_args[0]));
        assert_eq!(config.extra_protoc_args.get(1), Some(&extra_args[1]));
        Ok(())
    }

    fn quote(value: &str) -> String {
        ["\"", value, "\""].concat()
    }

    fn config_with_required_args<I, T>(additional_args: I) -> Result<Config>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let input_arg = arg(INPUT);
        let proto_arg = arg(PROTO);
        let mut args: Vec<String> = [APP_NAME, &input_arg, "path/to/input", &proto_arg, "cpp"]
            .map(|s| s.to_string())
            .into();
        for arg in additional_args {
            args.push(arg.into());
        }
        let config = Config::from_args(&parse_cli_args(args))?;
        Ok(config)
    }

    fn arg(name: &str) -> String {
        ["--", name].concat()
    }
}
