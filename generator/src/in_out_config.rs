use crate::util;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct InOutConfig {
    pub input: PathBuf,
    pub output: PathBuf,
}

impl InOutConfig {
    pub fn from_config(
        input: &str,
        output: &str,
        input_root: Option<&PathBuf>,
        output_root: Option<&PathBuf>,
    ) -> Result<Self> {
        Ok(InOutConfig {
            input: util::path_as_absolute(input, input_root)?,
            output: util::path_as_absolute(output, output_root)?,
        })
    }
}

#[cfg(test)]
mod tests {
    mod lang_config {
        use crate::in_out_config::InOutConfig;
        use crate::DisplayNormalized;
        use anyhow::Result;
        use std::env::current_dir;

        #[test]
        fn from_config_with_explicit_output() -> Result<()> {
            let input_path = current_dir()?;
            let output_path = current_dir()?;
            let config = InOutConfig::from_config(
                &input_path.display_normalized(),
                &output_path.display_normalized(),
                None,
                None,
            )?;
            assert_eq!(config.input, input_path);
            assert_eq!(config.output, output_path);
            Ok(())
        }
    }
}
