use crate::config::OUTPUT_SEPARATOR;
use crate::lang::Lang;
use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;

pub struct LangConfig {
    pub lang: Lang,
    pub output: PathBuf,
    pub output_prefix: PathBuf,
}

impl LangConfig {
    pub fn from_config(
        config: &str,
        output_root: &PathBuf,
        default_output_prefix: &str,
    ) -> Result<Self> {
        let (lang, path) = match config.split_once(OUTPUT_SEPARATOR) {
            None => (config, default_output(config, default_output_prefix)),
            Some((lang, path)) => (lang, path.into()),
        };
        let output_path = parse_output_path(output_root, &path)?;
        Ok(LangConfig {
            lang: Lang::from_str(lang)?,
            output: output_path.full(),
            output_prefix: output_path.prefix,
        })
    }
}

impl AsRef<LangConfig> for LangConfig {
    fn as_ref(&self) -> &LangConfig {
        &self
    }
}

fn default_output(config: &str, default_prefix: &str) -> PathBuf {
    PathBuf::from([default_prefix, config].join("_"))
}

struct OutputPath {
    pub prefix: PathBuf,
    pub path: PathBuf,
}

impl OutputPath {
    pub fn full(&self) -> PathBuf {
        self.prefix.join(&self.path)
    }
}

fn parse_output_path(output_root: &PathBuf, path: &PathBuf) -> Result<OutputPath> {
    if path.is_absolute() {
        return Ok(OutputPath {
            prefix: PathBuf::new(),
            path: path.clone(),
        });
    }
    let prefix = if path.is_absolute() {
        PathBuf::new()
    } else {
        output_root.clone()
    };
    Ok(OutputPath {
        prefix,
        path: path.clone(),
    })
}

#[cfg(test)]
mod tests {
    mod lang_config {
        use crate::config::{OUTPUT_SEPARATOR, PROTO};
        use crate::lang::Lang;
        use crate::lang_config::LangConfig;
        use anyhow::Result;
        use std::path::PathBuf;

        #[test]
        fn from_config_with_default_output() -> Result<()> {
            let output_root = PathBuf::new();
            let config = LangConfig::from_config(&Lang::CSharp.as_config(), &output_root, PROTO)?;
            assert_eq!(config.lang, Lang::CSharp);
            assert_eq!(config.output.as_path().to_str(), Some("proto_csharp"));
            Ok(())
        }

        #[test]
        fn from_config_with_explicit_output() -> Result<()> {
            let output_root = PathBuf::new();
            let output_path = PathBuf::from("path/to/output");
            let config =
                [&Lang::CSharp.as_config(), output_path.to_str().unwrap()].join(OUTPUT_SEPARATOR);
            let config = LangConfig::from_config(&config, &output_root, PROTO)?;
            assert_eq!(config.lang, Lang::CSharp);
            assert_eq!(config.output, output_path);
            Ok(())
        }

        #[test]
        fn from_config_with_unsupported_lang() {
            assert!(
                LangConfig::from_config("blah unsupported lang", &PathBuf::new(), PROTO).is_err()
            );
        }
    }

    mod parse_output_path {
        use crate::lang_config::parse_output_path;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn absolute_with_root() -> Result<()> {
            let root = env::current_dir()?;
            let path = env::temp_dir();
            let result = parse_output_path(&root, &path)?;
            assert_eq!(result.prefix, PathBuf::new());
            assert_eq!(result.path, path);
            assert_eq!(result.full(), path);
            Ok(())
        }

        #[test]
        fn relative_with_root() -> Result<()> {
            let root = env::temp_dir();
            let path = PathBuf::from("rel/path");
            let result = parse_output_path(&root, &path)?;
            assert_eq!(result.prefix, root);
            assert_eq!(result.path, path);
            assert_eq!(result.full(), root.join(path));
            Ok(())
        }
    }
}
