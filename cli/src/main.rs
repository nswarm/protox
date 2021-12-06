use crate::idl::Idl;
use crate::options::Options;
use crate::runner::Protoc;
use anyhow::Result;

mod idl;
mod options;
mod runner;

fn main() -> Result<()> {
    env_logger::init();

    let opt = Options::from_cli()?;
    let runner = match opt.idl {
        Idl::Proto => Protoc::with_options(opt),
    };
    runner.run()?;
    Ok(())
}
