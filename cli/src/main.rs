use crate::config::Config;
use crate::idl::Idl;
use anyhow::Result;

mod config;
mod idl;
mod lang;
mod lang_config;
mod run;

fn main() -> Result<()> {
    env_logger::init();

    let config = Config::from_cli()?;
    match config.idl {
        Idl::Proto => run::protoc(&config)?,
    };
    Ok(())
}
