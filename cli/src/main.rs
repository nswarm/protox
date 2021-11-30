use clap::{crate_version, App, Arg};
use log::info;
use std::env;
use std::process::Command;
use std::str::FromStr;

const IDL: &str = "idl";
const INPUT: &str = "input";
const OUTPUT_ROOT: &str = "output_root";
const PROTO: &str = "proto";
const SERVER: &str = "server";
const CLIENT: &str = "client";
const DIRECT: &str = "direct";
const OUTPUT_TYPES: [&str; 4] = [PROTO, SERVER, CLIENT, DIRECT];
const OUTPUT_VALUE_NAMES: [&str; 2] = ["name", "output"];
const PLUGIN_PROTO: &str = "plugin-proto";
const OUTPUT_LONG_ABOUT: & str = "When OUTPUT is a relative path, it is evaluated to either OUTPUT_ROOT if set, or the current working directory otherwise.";
const LONG_ABOUT_NEWLINE: &str = "\n\n";

fn main() {
    let _ = App::new("struct-ffi-gen")
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
        ])
        .get_matches();

    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let protoc_args = collect_protoc_args(&args);
    protoc(&protoc_args);
}

fn output_arg(name: &str) -> Arg {
    Arg::new(name)
        .default_short()
        .long(name)
        .required_unless_present_any(&OUTPUT_TYPES)
        .value_names(&OUTPUT_VALUE_NAMES)
}

#[derive(clap::ArgEnum, Clone, Debug)]
enum Idl {
    Proto,
}

impl FromStr for Idl {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proto" => Ok(Idl::Proto),
            _ => Err("Invalid IDL"),
        }
    }
}

impl Idl {
    fn as_config(&self) -> String {
        match self {
            Idl::Proto => "proto",
        }
        .to_string()
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

fn collect_protoc_args(args: &[String]) -> Vec<String> {
    // Slice off the first arg (typically exe name) and pass all actual args to protoc.
    args[1..].to_vec()
}

fn protoc(args: &[String]) {
    let protoc_path = prost_build::protoc();
    info!("located protoc: {:?}", protoc_path);
    info!("running:\nprotoc {:?}", args.join(" "));
    let mut child = Command::new(protoc_path)
        .args(args)
        .spawn()
        .expect("Failed to execute protoc");
    match child.wait() {
        Ok(status) => {
            if !status.success() {
                println!("Exited with status {}", status);
            }
        }
        Err(err) => println!("Exited with error {}", err),
    }
}
