use std::fs;
use crate::Config;
use anyhow::{Context, Result};

mod input;
mod proto;
mod direct;
mod client;
mod server;
mod util;

pub use proto::supported_languages as proto_supported_languages;
pub use direct::supported_languages as direct_supported_languages;
pub use client::supported_languages as client_supported_languages;
pub use server::supported_languages as server_supported_languages;

pub fn configured(config: &Config) -> Result<()> {
    let input_files = input::collect(config).context("Failed to collect input files.")?;
    create_output_dirs(config)?;
    proto::run(config, &input_files)?;
    direct::run(config, &input_files)?;
    client::run(config, &input_files)?;
    server::run(config, &input_files)?;
    Ok(())
}

fn create_output_dirs(config: &Config) -> Result<()> {
    for proto in &config.proto {
        fs::create_dir_all(&proto.output).with_context(|| {
            format!(
                "Failed to create directory at path {:?} for proto output '{}'",
                proto.output,
                proto.lang.as_config()
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use anyhow::Result;
    use tempfile::tempdir;
    use crate::{Config, Lang};
    use crate::lang_config::LangConfig;
    use crate::run::create_output_dirs;

    #[test]
    fn create_output_dirs_test() -> Result<()> {
        let tempdir = tempdir()?;
        let root = tempdir.path();
        let mut config = Config::default();
        config.proto.push(lang_config_with_output(Lang::Cpp, root));
        config.proto.push(lang_config_with_output(Lang::CSharp, root));
        create_output_dirs(&config)?;
        assert!(fs::read_dir(lang_path(Lang::Cpp, root)).is_ok());
        assert!(fs::read_dir(lang_path(Lang::CSharp, root)).is_ok());
        Ok(())
    }

    fn lang_config_with_output(lang: Lang, root: &Path) -> LangConfig {
        LangConfig {
            lang: lang.clone(),
            output: lang_path( lang, root),
            output_prefix: PathBuf::new(),
        }
    }

    fn lang_path(lang: Lang, root: &Path) -> PathBuf {
        root.join(lang.as_config())
    }
}
