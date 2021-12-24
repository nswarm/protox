use crate::util;
use anyhow::Result;
use std::path::PathBuf;

pub struct TemplateConfig {
    pub input: PathBuf,
    pub output: PathBuf,
}

impl TemplateConfig {
    pub fn from_config(
        input: &str,
        output: &str,
        template_root: Option<&PathBuf>,
        output_root: Option<&PathBuf>,
    ) -> Result<Self> {
        Ok(TemplateConfig {
            input: util::parse_rooted_path(template_root, input, "template")?,
            output: util::parse_rooted_path(output_root, output, "output")?,
        })
    }
}

// todo do I need this?
impl AsRef<TemplateConfig> for TemplateConfig {
    fn as_ref(&self) -> &TemplateConfig {
        &self
    }
}

#[cfg(test)]
mod tests {
    mod lang_config {
        use crate::template_config::TemplateConfig;
        use crate::DisplayNormalized;
        use anyhow::Result;
        use std::env::current_dir;

        #[test]
        fn from_config_with_explicit_output() -> Result<()> {
            let input_path = current_dir()?;
            let output_path = current_dir()?;
            let config = TemplateConfig::from_config(
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
