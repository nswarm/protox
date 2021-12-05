use crate::idl::Idl;
use anyhow::Result;
use clap::{crate_version, App, Arg, ArgMatches};
use std::path::PathBuf;
use thiserror::Error;

pub const IDL: &str = "idl";
pub const INPUT: &str = "input";
pub const OUTPUT_ROOT: &str = "output_root";
pub const PROTO: &str = "proto";
pub const SERVER: &str = "server";
pub const CLIENT: &str = "client";
pub const DIRECT: &str = "direct";
pub const OUTPUT_TYPES: [&str; 4] = [PROTO, SERVER, CLIENT, DIRECT];
pub const OUTPUT_VALUE_NAMES: [&str; 2] = ["name", "output"];
pub const PLUGIN_PROTO: &str = "plugin-proto";
pub const OUTPUT_LONG_ABOUT: & str = "When OUTPUT is a relative path, it is evaluated to either OUTPUT_ROOT if set, or the current working directory otherwise.";
pub const LONG_ABOUT_NEWLINE: &str = "\n\n";

fn parse_cli_args() -> ArgMatches {
    App::new("struct-ffi-gen")
        .long_about("struct-ffi-gen is an executable that generates C-ABI-compatible code in one or more languages for seamless and performant direct usage of those types across the library boundary.")
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
                .long(OUTPUT_ROOT),

            output_arg(PROTO)
                .display_order(100)
                .about("")
                .long_about(&[
                    "Indicates protobuf code should be generated to file path OUTPUT.",
                    OUTPUT_LONG_ABOUT,
                    "NAME indicates the name of the language to generate code for.",
                    &format!("Supported languages: All languages specified in your `protoc --help` as *_out args. Additionally 'rust' is supported. \
                    Custom support can be added via the used of {}.", PLUGIN_PROTO),
                ].join(LONG_ABOUT_NEWLINE)),

            output_arg(SERVER)
                .display_order(101),

            output_arg(CLIENT)
                .display_order(102),

            output_arg(DIRECT)
                .display_order(103),
        ]).get_matches()
}

fn output_arg(name: &str) -> Arg {
    Arg::new(name)
        .default_short()
        .long(name)
        .required_unless_present_any(&OUTPUT_TYPES)
        .value_names(&OUTPUT_VALUE_NAMES)
}

#[derive(Error, Debug)]
enum ParseError {}

pub struct Options {
    // todo prob make these pub (?)
    idl: Idl,
    input: PathBuf,
    output_root: PathBuf,
}

impl Options {
    pub fn new() -> Result<Self> {
        let args = parse_cli_args();
        let options = Options::from_args(&args)?;
        Ok(options)
    }

    pub fn idl(&self) -> &Idl {
        &self.idl
    }

    pub fn input(&self) -> &PathBuf {
        &self.input
    }

    pub fn output_root(&self) -> &PathBuf {
        &self.output_root
    }

    fn from_args(args: &ArgMatches) -> Result<Self> {
        Ok(Self {
            idl: Idl::from_args(&args)?,
            input: PathBuf::new(),
            output_root: PathBuf::new(),
        })
    }
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
