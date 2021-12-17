use anyhow::anyhow;
use std::str::FromStr;

#[derive(clap::ArgEnum, Clone, Debug, Eq, PartialEq)]
pub enum Lang {
    Cpp,
    CSharp,
    Java,
    Javascript,
    Kotlin,
    ObjectiveC,
    Php,
    Python,
    Ruby,
    Rust,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::Cpp
    }
}

impl FromStr for Lang {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &s.to_lowercase()[..] {
            "cpp" => Lang::Cpp,
            "csharp" => Lang::CSharp,
            "java" => Lang::Java,
            "js" => Lang::Javascript,
            "kotlin" => Lang::Kotlin,
            "objc" => Lang::ObjectiveC,
            "php" => Lang::Php,
            "python" => Lang::Python,
            "ruby" => Lang::Ruby,
            "rust" => Lang::Rust,
            _ => return Err(anyhow!("Unsupported Language: {}", s)),
        })
    }
}

impl Lang {
    pub fn as_config(&self) -> String {
        match self {
            Lang::Cpp => "cpp",
            Lang::CSharp => "csharp",
            Lang::Java => "java",
            Lang::Javascript => "js",
            Lang::Kotlin => "kotlin",
            Lang::ObjectiveC => "objc",
            Lang::Php => "php",
            Lang::Python => "python",
            Lang::Ruby => "ruby",
            Lang::Rust => "rust",
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
