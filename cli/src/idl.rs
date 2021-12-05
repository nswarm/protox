use crate::options;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::str::FromStr;

#[derive(clap::ArgEnum, Clone, Debug, Eq, PartialEq)]
pub enum Idl {
    Proto,
}

impl Idl {
    pub fn from_args(args: &ArgMatches) -> Result<Self> {
        let idl_str = args
            .value_of(options::IDL)
            .expect("IDL missing default value.");
        Idl::from_str(idl_str)
    }
}

impl FromStr for Idl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proto" => Ok(Idl::Proto),
            _ => Err(anyhow!("Unsupported IDL: {}", s)),
        }
    }
}

impl Idl {
    pub(crate) fn as_config(&self) -> String {
        match self {
            Idl::Proto => "proto",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::idl::Idl;
    use anyhow::Result;
    use std::str::FromStr;

    #[test]
    fn proto() -> Result<()> {
        let idl_str = Idl::from_str(&Idl::Proto.as_config())?;
        assert_eq!(idl_str, Idl::Proto);
        Ok(())
    }
}
