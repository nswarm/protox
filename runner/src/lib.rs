use anyhow::Result;

mod config;
mod generator;
mod idl;
mod lang;
mod lang_config;
mod protoc;
mod util;

pub use config::Config;
pub use idl::Idl;
pub use lang::Lang;

pub fn run() -> Result<()> {
    env_logger::init();
    let config = Config::from_cli()?;
    run_internal(config)
}

pub fn run_with_config(config: Config) -> Result<()> {
    env_logger::init();
    run_internal(config)
}

fn run_internal(config: Config) -> Result<()> {
    match config.idl {
        Idl::Proto => {
            protoc::generate_descriptor_set_and_builtin_lang_outputs(&config)?;
            generator::generate(&config)?;
        }
    };
    Ok(())
}
