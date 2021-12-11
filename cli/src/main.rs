use crate::idl::Idl;
use crate::options::Options;
use anyhow::Result;

mod idl;
mod lang;
mod options;
mod run;

fn main() -> Result<()> {
    env_logger::init();

    let options = Options::from_cli()?;
    match options.idl {
        Idl::Proto => run::protoc(&options)?,
    };
    Ok(())
}
