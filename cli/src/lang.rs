use anyhow::anyhow;
use std::str::FromStr;

#[derive(clap::ArgEnum, Clone, Debug, Eq, PartialEq)]
pub enum Lang {
    Unsupported,
    CSharp,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::Unsupported
    }
}

impl FromStr for Lang {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "csharp" => Ok(Lang::CSharp),
            _ => Err(anyhow!("Unsupported Language: {}", s)),
        }
    }
}

impl Lang {
    pub(crate) fn as_config(&self) -> String {
        match self {
            Lang::CSharp => "csharp",
            Lang::Unsupported => "unsupported",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::Lang;
    use anyhow::Result;
    use std::str::FromStr;

    #[test]
    fn proto() -> Result<()> {
        let lang_str = Lang::from_str(&Lang::CSharp.as_config())?;
        assert_eq!(lang_str, Lang::CSharp);
        Ok(())
    }
}
