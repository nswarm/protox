use crate::lang::Lang;
use crate::options::{OUTPUT_ROOT, OUTPUT_SEPARATOR};
use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

pub struct LangOption {
    pub lang: Lang,
    pub output: PathBuf,
}

impl LangOption {
    pub fn from_config(
        config: &str,
        output_root: Option<&PathBuf>,
        default_output_prefix: &str,
    ) -> Result<Self> {
        let (lang, path) = match config.split_once(OUTPUT_SEPARATOR) {
            None => (config, default_output(config, default_output_prefix)),
            Some((lang, path)) => (lang, path.into()),
        };
        Ok(LangOption {
            lang: Lang::from_str(lang)?,
            output: parse_output_path(output_root, &path)?,
        })
    }
}

fn default_output(config: &str, default_prefix: &str) -> PathBuf {
    PathBuf::from([default_prefix, config].join("_"))
}

fn parse_output_path(output_root: Option<&PathBuf>, path: &PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.clone());
    }
    match output_root {
        None => Ok(env::current_dir()
            .with_context(|| {
                format!(
                    "Working directory does not exist or permission denied.\
                     Try specifying an explicit --{} or running from a different folder.",
                    OUTPUT_ROOT
                )
            })?
            .join(path)),
        Some(root) => Ok(root.join(path)),
    }
}

#[cfg(test)]
mod tests {
    mod lang_option {
        use crate::lang::Lang;
        use crate::lang_option::LangOption;
        use crate::options::{OUTPUT_SEPARATOR, PROTO};
        use anyhow::Result;
        use std::path::PathBuf;

        #[test]
        fn from_config_with_default_output() -> Result<()> {
            let output_root = PathBuf::new();
            let option =
                LangOption::from_config(&Lang::CSharp.as_config(), Some(&output_root), PROTO)?;
            assert_eq!(option.lang, Lang::CSharp);
            assert_eq!(option.output.as_path().to_str(), Some("proto_csharp"));
            Ok(())
        }

        #[test]
        fn from_config_with_explicit_output() -> Result<()> {
            let output_root = PathBuf::new();
            let output_path = PathBuf::from("path/to/output");
            let config =
                [&Lang::CSharp.as_config(), output_path.to_str().unwrap()].join(OUTPUT_SEPARATOR);
            let option = LangOption::from_config(&config, Some(&output_root), PROTO)?;
            assert_eq!(option.lang, Lang::CSharp);
            assert_eq!(option.output, output_path);
            Ok(())
        }

        #[test]
        fn from_config_with_unsupported_lang() {
            assert!(LangOption::from_config("blah unsupported lang", None, PROTO).is_err());
        }
    }

    mod parse_output_path {
        use crate::lang_option::parse_output_path;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn absolute_with_root() -> Result<()> {
            let root = env::current_dir()?;
            let path = env::temp_dir();
            let result = parse_output_path(Some(&root), &path)?;
            assert_eq!(result, path);
            Ok(())
        }

        #[test]
        fn absolute_no_root() -> Result<()> {
            let path = env::temp_dir();
            let result = parse_output_path(None, &path)?;
            assert_eq!(result, path);
            Ok(())
        }

        #[test]
        fn relative_with_root() -> Result<()> {
            let root = env::temp_dir();
            let path = PathBuf::from("rel/path");
            let result = parse_output_path(Some(&root), &path)?;
            assert_eq!(result, root.join(path));
            Ok(())
        }

        #[test]
        fn relative_no_root() -> Result<()> {
            let path = PathBuf::from("rel/path");
            let result = parse_output_path(None, &path)?;
            assert_eq!(result, env::current_dir()?.join(path));
            Ok(())
        }
    }
}
