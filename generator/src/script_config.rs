use crate::{util, InOutConfig};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ScriptConfig {
    pub name: String,
    pub input: PathBuf,
    pub output: PathBuf,
    pub overlays: Vec<PathBuf>,
}

impl ScriptConfig {
    pub fn from_config(
        name: &str,
        script_in: Option<&str>,
        script_out: Option<&str>,
        script_root: Option<&PathBuf>,
        output_root: Option<&PathBuf>,
        overlays: &[&str],
    ) -> Result<Self> {
        Ok(ScriptConfig {
            name: name.to_string(),
            input: util::path_as_absolute(script_in.unwrap_or(name), script_root)?,
            output: util::path_as_absolute(script_out.unwrap_or(name), output_root)?,
            overlays: overlays
                .iter()
                .filter_map(|x| util::path_as_absolute(x, script_root).ok())
                .collect::<Vec<PathBuf>>(),
        })
    }
}

impl From<ScriptConfig> for InOutConfig {
    fn from(x: ScriptConfig) -> Self {
        InOutConfig {
            input: x.input,
            output: x.output,
            overlays: x.overlays,
        }
    }
}

impl From<&ScriptConfig> for InOutConfig {
    fn from(x: &ScriptConfig) -> Self {
        InOutConfig {
            input: x.input.clone(),
            output: x.output.clone(),
            overlays: x.overlays.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::script_config::ScriptConfig;
    use anyhow::Result;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn name() -> Result<()> {
        let config = ScriptConfig::from_config(
            "name",
            None,
            None,
            Some(&PathBuf::new()),
            Some(&PathBuf::new()),
            &[],
        )?;
        assert_eq!(&config.name, &"name");
        Ok(())
    }

    mod input {
        use crate::script_config::ScriptConfig;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn in_only() -> Result<()> {
            let dir = env::current_dir()?;
            let config = ScriptConfig::from_config(
                "name",
                Some(dir.to_str().unwrap()),
                None,
                None,
                Some(&PathBuf::new()),
                &[],
            )?;
            assert_eq!(config.input, dir);
            Ok(())
        }

        #[test]
        fn root_only() -> Result<()> {
            let dir = env::current_dir()?;
            let config = ScriptConfig::from_config(
                "name",
                None,
                None,
                Some(&dir),
                Some(&PathBuf::new()),
                &[],
            )?;
            assert_eq!(config.input, dir.join(&config.name));
            Ok(())
        }

        #[test]
        fn root_and_in() -> Result<()> {
            let dir = env::current_dir()?;
            let config = ScriptConfig::from_config(
                "name",
                Some("other"),
                None,
                Some(&dir),
                Some(&PathBuf::new()),
                &[],
            )?;
            assert_eq!(config.input, dir.join("other"));
            Ok(())
        }
    }

    mod output {
        use crate::script_config::ScriptConfig;
        use anyhow::Result;
        use std::env;
        use std::path::PathBuf;

        #[test]
        fn out_only() -> Result<()> {
            let dir = env::current_dir()?;
            let config = ScriptConfig::from_config(
                "name",
                None,
                Some(dir.to_str().unwrap()),
                Some(&PathBuf::new()),
                Some(&PathBuf::new()),
                &[],
            )?;
            assert_eq!(config.output, dir);
            Ok(())
        }

        #[test]
        fn root_only() -> Result<()> {
            let dir = env::current_dir()?;
            let config = ScriptConfig::from_config(
                "name",
                None,
                None,
                Some(&PathBuf::new()),
                Some(&dir),
                &[],
            )?;
            assert_eq!(config.output, dir.join(&config.name));
            Ok(())
        }

        #[test]
        fn root_and_out() -> Result<()> {
            let dir = env::current_dir()?;
            let config = ScriptConfig::from_config(
                "name",
                None,
                Some("other"),
                Some(&PathBuf::new()),
                Some(&dir),
                &[],
            )?;
            assert_eq!(config.output, dir.join("other"));
            Ok(())
        }
    }

    #[test]
    fn overlays() -> Result<()> {
        let dir = env::current_dir()?;
        let config = ScriptConfig::from_config(
            "name",
            None,
            None,
            Some(&dir),
            Some(&PathBuf::new()),
            &["a", dir.join("test").join("b").to_str().unwrap(), "c"],
        )?;
        assert_eq!(
            config.overlays,
            vec![dir.join("a"), dir.join("test").join("b"), dir.join("c"),]
        );
        Ok(())
    }
}
