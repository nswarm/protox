use anyhow::Result;

mod config;
mod idl;
mod lang;
mod lang_config;
mod run;

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
        Idl::Proto => run::configured(&config)?,
    };
    Ok(())
}
