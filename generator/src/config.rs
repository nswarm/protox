use crate::idl::Idl;
use crate::in_out_config::InOutConfig;
use crate::lang::Lang;
use crate::lang_config::LangConfig;
use crate::protoc;
use anyhow::{anyhow, Context, Result};
use clap::{crate_version, App, Arg, ArgMatches, Values};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

pub const APP_NAME: &str = "protox";
pub const IDL: &str = "idl";
pub const INPUT: &str = "input";
pub const PROTO: &str = "proto";
pub const SCRIPT: &str = "script";
pub const TEMPLATE: &str = "template";
pub const BYPASS: &str = "bypass";
pub const TEMPLATE_ROOT: &str = "template-root";
pub const SCRIPT_ROOT: &str = "script-root";
pub const OUTPUT_ROOT: &str = "output-root";
pub const INCLUDES: &str = "includes";
pub const INIT_SCRIPT: &str = "init-script";
pub const INIT_TEMPLATE: &str = "init-template";
pub const DESCRIPTOR_SET_OUT: &str = "descriptor-set-out";
pub const PROTOC_ARGS: &str = "protoc-args";
pub const LONG_HELP_NEWLINE: &str = "\n\n";

const MAIN_OPTS: &[&str; 6] = &[PROTO, TEMPLATE, SCRIPT, BYPASS, INIT_SCRIPT, INIT_TEMPLATE];

const DISPLAY_ORDER_DEFAULT: usize = 990;
const DEFAULT_DESCRIPTOR_SET_FILENAME: &str = "descriptor_set.proto";

fn parse_cli_args<I, T>(iter: I) -> Result<ArgMatches, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut i = 0;
    let mut display_order = || {
        i = i + 1;
        i
    };
    App::new(APP_NAME)
        .long_about("protox is an executable that wraps the protobuf compiler (protoc) with a simpler to use interface that makes it easier to write and use custom code generator plugins.")
        .version(crate_version!())
        .args([
            Arg::new(IDL)
                .display_order(display_order())
                .help("IDL type of files expected at the INPUT path.")
                .long(IDL)
                .default_value(&Idl::Proto.as_config())
                // Only 1 option atm, not useful.
                .hide(true),

            Arg::new(INPUT)
                .display_order(1)
                .help("File path to search for protobuf IDL files.")
                .default_short()
                .long(INPUT)
                .takes_value(true)
                .required(true)
                .conflicts_with_all(&[INIT_SCRIPT, INIT_TEMPLATE]),

            Arg::new(PROTO)
                .display_order(display_order())
                .long_help(join_help(&[
                    "Protobuf code will be generated for language LANG to directory located at OUTPUT.",
                    &format!("If OUTPUT is a relative path, it is evaluated relative to --{}.", OUTPUT_ROOT),
                    &format!("Supported languages for LANG: {}.", lang_list(&protoc::supported_languages())),
                ]).as_str())
                .default_short()
                .long(PROTO)
                .value_names(&["LANG", "OUTPUT"])
                .multiple_occurrences(true)
                .required_unless_present_any(all_except(MAIN_OPTS, PROTO))
                .conflicts_with_all(&[INIT_SCRIPT, INIT_TEMPLATE]),

            Arg::new(TEMPLATE)
                .display_order(display_order())
                .long_help(join_help(&[
                    "Code will be generated for the templates and configuration found inside the INPUT folder, and written to directory located at OUTPUT.",
                    "Templates use the mustache template language (https://mustache.github.io/).",
                    &format!("If INPUT is a relative path, it is evaluated relative to --{}.", TEMPLATE_ROOT),
                    &format!("If OUTPUT is a relative path, it is evaluated relative to --{}.", OUTPUT_ROOT),
                    "See the examples folder for how to set up the INPUT directory correctly.",
                ]).as_str())
                .default_short()
                .long(TEMPLATE)
                .value_names(&["INPUT", "OUTPUT"])
                .multiple_occurrences(true)
                .required_unless_present_any(all_except(MAIN_OPTS, TEMPLATE))
                .conflicts_with_all(&[INIT_SCRIPT, INIT_TEMPLATE]),

            Arg::new(SCRIPT)
                .display_order(display_order())
                .long_help(join_help(&[
                    "Code will be generated for the scripts found inside the INPUT folder, and written to directory located at OUTPUT.",
                    "Scripts use the language rhai (https://rhai.rs/).",
                    &format!("If INPUT is a relative path, it is evaluated relative to --{}.", TEMPLATE_ROOT),
                    &format!("If OUTPUT is a relative path, it is evaluated relative to --{}.", OUTPUT_ROOT),
                    "See the examples folder for how to set up the INPUT directory correctly.",
                ]).as_str())
                .default_short()
                .long(SCRIPT)
                .value_names(&["INPUT", "OUTPUT"])
                .multiple_occurrences(true)
                .required_unless_present_any(all_except(MAIN_OPTS, SCRIPT))
                .conflicts_with_all(&[INIT_SCRIPT, INIT_TEMPLATE]),

            Arg::new(BYPASS)
                .display_order(display_order())
                .long_help("Bypass protox additional functionality and run protoc directly.")
                .default_short()
                .long(BYPASS)
                .multiple_occurrences(true)
                .required_unless_present_any(all_except(MAIN_OPTS, BYPASS))
                .conflicts_with(INIT_SCRIPT)
                .conflicts_with_all(&[INIT_SCRIPT, INIT_TEMPLATE])
                .conflicts_with_all(&all_except(MAIN_OPTS, BYPASS)),

            Arg::new(TEMPLATE_ROOT)
                .display_order(display_order())
                .help(format!("All non-absolute --{} INPUT paths will be prefixed with this path. Required if any --{} INPUT paths are relative.", TEMPLATE, TEMPLATE).as_str())
                .long(TEMPLATE_ROOT)
                .takes_value(true),

            Arg::new(SCRIPT_ROOT)
                .display_order(display_order())
                .help(format!("All non-absolute --{} INPUT paths will be prefixed with this path. Required if any --{} INPUT paths are relative.", SCRIPT, SCRIPT).as_str())
                .long(SCRIPT_ROOT)
                .takes_value(true),

            Arg::new(OUTPUT_ROOT)
                .display_order(display_order())
                .help("All non-absolute output paths will be prefixed with this path. Required if any OUTPUT paths are relative.")
                .default_short()
                .long(OUTPUT_ROOT)
                .takes_value(true),

            Arg::new(INCLUDES)
                .display_order(display_order())
                .help("Additional include folders passed directly to protoc as --proto_path options.")
                .long(INCLUDES)
                .takes_value(true)
                .multiple_values(true),

            Arg::new(INIT_SCRIPT)
                .display_order(display_order())
                .help(format!("Initialize the TARGET directory as a new scripted rendering target with the basic input files required for running protox with --{}.", SCRIPT).as_str())
                .long(INIT_SCRIPT)
                .takes_value(true)
                .value_name("TARGET")
                .conflicts_with(INIT_TEMPLATE),

            Arg::new(INIT_TEMPLATE)
                .display_order(display_order())
                .help(format!("Initialize the TARGET directory as a new template rendering target with the basic input files required for running protox with --{}.", TEMPLATE).as_str())
                .long(INIT_TEMPLATE)
                .takes_value(true)
                .value_name("TARGET")
                .conflicts_with(INIT_SCRIPT),

            Arg::new(DESCRIPTOR_SET_OUT)
                .display_order(DISPLAY_ORDER_DEFAULT)
                .default_value(DEFAULT_DESCRIPTOR_SET_FILENAME)
                .long_help(join_help(&[
                    "Absolute output path for the descriptor_set proto file generated by protoc. By default it will be created in a temp folder that is deleted after the program is finished running.",
                    "This file is used by the generators other than those built into protoc itself.",
                ]).as_str())
                .long(DESCRIPTOR_SET_OUT)
                .takes_value(true),

            Arg::new(PROTOC_ARGS)
                .display_order(DISPLAY_ORDER_DEFAULT)
                .long_help(format!("Add any arguments directly to protoc invocation. Note they must be wrapped with \"\" as to not be picked up as arguments to protox.\nFor example: --{} \"--error_format=FORMAT\"", PROTOC_ARGS).as_str())
                .long(PROTOC_ARGS)
                .takes_value(true)
                .multiple_values(true),

        ]).try_get_matches_from(iter)
}

fn join_help(lines: &[&str]) -> String {
    lines.join(LONG_HELP_NEWLINE).to_owned()
}

fn all_except<'a>(options: &[&'a str], opt: &'a str) -> Vec<&'a str> {
    let mut v = options.to_vec();
    v.retain(|x| *x != opt);
    v
}

pub struct Config {
    pub idl: Idl,
    pub input: PathBuf,
    pub protos: Vec<LangConfig>,
    pub templates: Vec<InOutConfig>,
    pub scripts: Vec<InOutConfig>,
    pub bypass: bool,
    pub includes: Vec<String>,
    pub init_script_target: Option<PathBuf>,
    pub init_template_target: Option<PathBuf>,
    pub descriptor_set_path: PathBuf,
    pub extra_protoc_args: Vec<String>,

    // Owned here to keep alive for full program execution.
    #[allow(dead_code)]
    intermediate_dir: TempDir,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            idl: Default::default(),
            input: Default::default(),
            protos: vec![],
            templates: vec![],
            scripts: vec![],
            bypass: false,
            includes: vec![],
            init_script_target: None,
            init_template_target: None,
            descriptor_set_path: Default::default(),
            extra_protoc_args: vec![],
            intermediate_dir: tempdir().unwrap(),
        }
    }
}

impl Config {
    pub fn from_cli() -> Result<Self> {
        let args = parse_cli_args(&mut env::args_os()).unwrap_or_else(|e| e.exit());
        let config = Config::from_args(&args)?;
        check_proto_supported_languages(&config)?;
        Ok(config)
    }

    pub fn from_args(args: &ArgMatches) -> Result<Self> {
        let intermediate_dir = tempdir()?;
        let input = parse_optional_path_from_arg(INPUT, &args)?.unwrap_or(PathBuf::new());
        let output_root = parse_optional_path_from_arg(OUTPUT_ROOT, &args)?;
        let template_root = parse_optional_path_from_arg(TEMPLATE_ROOT, &args)?;
        let script_root = parse_optional_path_from_arg(SCRIPT_ROOT, &args)?;
        let descriptor_set_path = parse_descriptor_path(intermediate_dir.path(), &args);
        let config = Self {
            idl: Idl::from_args(&args)?,
            input,
            protos: parse_protos(&args, output_root.as_ref())?,
            templates: parse_in_out_configs(
                TEMPLATE,
                &args,
                template_root.as_ref(),
                output_root.as_ref(),
            )?,
            scripts: parse_in_out_configs(
                SCRIPT,
                &args,
                script_root.as_ref(),
                output_root.as_ref(),
            )?,
            bypass: args.is_present(BYPASS),
            includes: parse_includes(&args),
            init_script_target: parse_optional_path_from_arg(INIT_SCRIPT, &args)?,
            init_template_target: parse_optional_path_from_arg(INIT_TEMPLATE, &args)?,
            descriptor_set_path,
            extra_protoc_args: parse_extra_protoc_args(&args),
            intermediate_dir,
        };
        check_proto_supported_languages(&config)?;
        Ok(config)
    }
}

fn check_proto_supported_languages(config: &Config) -> Result<()> {
    check_supported_languages(PROTO, &config.protos, &protoc::supported_languages())
}

fn check_supported_languages(
    name: &str,
    config: &Vec<LangConfig>,
    supported_languages: &[Lang],
) -> Result<()> {
    for lang_config in config {
        if !supported_languages.contains(&lang_config.lang) {
            return Err(anyhow!(
                "Language `{}` is not supported for {} generation.",
                lang_config.lang.as_config(),
                name
            ));
        }
    }
    Ok(())
}

fn parse_optional_path_from_arg(arg_name: &str, args: &ArgMatches) -> Result<Option<PathBuf>> {
    match args.value_of(arg_name) {
        None => Ok(None),
        Some(input) => Ok(Some(current_dir(arg_name)?.join(input))),
    }
}

fn current_dir(arg_name: &str) -> Result<PathBuf> {
    env::current_dir().with_context(|| {
        format!(
            "Working directory does not exist or permission denied.\
                     Try specifying an explicit --{} or running from a different folder.",
            arg_name
        )
    })
}

fn parse_descriptor_path(intermediate_dir: &Path, args: &ArgMatches) -> PathBuf {
    intermediate_dir.join(
        args.value_of(DESCRIPTOR_SET_OUT)
            .unwrap_or(DEFAULT_DESCRIPTOR_SET_FILENAME),
    )
}

fn parse_protos(args: &ArgMatches, output_root: Option<&PathBuf>) -> Result<Vec<LangConfig>> {
    let mut configs = Vec::new();
    let values = match args.grouped_values_of(PROTO) {
        None => return Ok(configs),
        Some(values) => values,
    };
    for value in values {
        let lang = value.get(0).ok_or(anyhow!("--{} is missing LANG", PROTO))?;
        let output = value
            .get(1)
            .ok_or(anyhow!("--{} is missing OUTPUT", PROTO))?;
        configs.push(LangConfig::from_config(lang, output, output_root)?);
    }
    Ok(configs)
}

fn parse_in_out_configs(
    arg_name: &str,
    args: &ArgMatches,
    input_root: Option<&PathBuf>,
    output_root: Option<&PathBuf>,
) -> Result<Vec<InOutConfig>> {
    let mut configs = Vec::new();
    let values = match args.grouped_values_of(arg_name) {
        None => return Ok(configs),
        Some(values) => values,
    };
    for value in values {
        let input = value
            .get(0)
            .ok_or(anyhow!("--{} is missing INPUT", arg_name))?;
        let output = value
            .get(1)
            .ok_or(anyhow!("--{} is missing OUTPUT", arg_name))?;
        configs.push(InOutConfig::from_config(
            input,
            output,
            input_root,
            output_root,
        )?);
    }
    Ok(configs)
}

fn parse_includes(args: &ArgMatches) -> Vec<String> {
    parse_arg_to_vec(INCLUDES, args)
}

fn parse_extra_protoc_args(args: &ArgMatches) -> Vec<String> {
    parse_arg_to_vec(PROTOC_ARGS, args)
}

fn parse_arg_to_vec(arg_name: &str, args: &ArgMatches) -> Vec<String> {
    args.values_of(arg_name)
        .unwrap_or(Values::default())
        .map(&str::to_owned)
        .collect()
}

fn lang_list(list: &[Lang]) -> String {
    list.iter()
        .map(|lang| lang.as_config())
        .collect::<Vec<String>>()
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
    use crate::config::{
        parse_cli_args, APP_NAME, INCLUDES, INPUT, OUTPUT_ROOT, PROTO, PROTOC_ARGS,
    };
    use crate::{Config, DisplayNormalized};
    use anyhow::Result;
    use std::env::current_dir;

    #[test]
    fn parse_input() -> Result<()> {
        let input = current_dir()?;
        let output = current_dir()?;
        let config = Config::from_args(&parse_cli_args([
            APP_NAME,
            &arg(INPUT),
            &input.display_normalized(),
            &arg(PROTO),
            "cpp",
            "proto_cpp",
            &arg(OUTPUT_ROOT),
            &output.display_normalized(),
        ])?)?;
        assert_eq!(config.input, input);
        Ok(())
    }

    mod parse_descriptor_path {
        use crate::config::tests::{arg, config_with_required_args};
        use crate::config::{DEFAULT_DESCRIPTOR_SET_FILENAME, DESCRIPTOR_SET_OUT};
        use anyhow::Result;

        #[test]
        fn default() -> Result<()> {
            let config = config_with_required_args(Vec::<String>::new())?;
            let intermediate_dir = config.intermediate_dir.path();
            assert_eq!(
                config.descriptor_set_path,
                intermediate_dir.join(DEFAULT_DESCRIPTOR_SET_FILENAME),
            );
            Ok(())
        }

        #[test]
        fn explicit() -> Result<()> {
            let explicit_path = "path/to/desc/set";
            let arg = arg(DESCRIPTOR_SET_OUT);
            let config = config_with_required_args([&arg, explicit_path])?;
            assert_eq!(
                config.descriptor_set_path,
                config.intermediate_dir.path().join(explicit_path)
            );
            Ok(())
        }
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

    #[test]
    fn parse_includes() -> Result<()> {
        let includes = [quote("include0"), quote("include1")];
        let arg = arg(INCLUDES);
        let config = config_with_required_args([&arg, &includes[0], &includes[1]])?;
        assert_eq!(config.includes.get(0), Some(&includes[0]));
        assert_eq!(config.includes.get(1), Some(&includes[1]));
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
        let mut args: Vec<String> = [
            APP_NAME,
            &input_arg,
            "path/to/input",
            &proto_arg,
            "cpp",
            &current_dir()?.join("proto_cpp").display_normalized(),
        ]
        .map(|s| s.to_owned())
        .into();
        for arg in additional_args {
            args.push(arg.into());
        }
        let config = Config::from_args(&parse_cli_args(args)?)?;
        Ok(config)
    }

    fn arg(name: &str) -> String {
        ["--", name].concat()
    }
}
