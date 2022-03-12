#![forbid(unsafe_code)]

mod config;
mod extension_registry;
mod idl;
mod in_out_config;
mod in_out_generator;
mod lang;
mod lang_config;
mod protoc;
mod render;
mod scripted_renderer;
mod template_init;
mod template_renderer;
mod util;

use crate::template_init::initialize_template_dir;
use crate::template_renderer::CONFIG_FILE_NAME;
use crate::util::DisplayNormalized;
use anyhow::Result;
pub use config::Config;
pub use idl::Idl;
pub use in_out_config::InOutConfig;
pub use lang::Lang;
pub use lang_config::LangConfig;

pub fn generate() -> Result<()> {
    env_logger::init();
    let config = Config::from_cli()?;
    generate_internal(&config)
}

pub fn generate_with_config(config: Config) -> Result<()> {
    env_logger::init();
    generate_internal(&config)
}

fn generate_internal(config: &Config) -> Result<()> {
    if let Some(init_target) = &config.init_target {
        return initialize_template_dir(&init_target);
    }
    match config.idl {
        Idl::Proto => {
            protoc::generate(&config)?;
            template_renderer::generate(&config)?;
            scripted_renderer::generate(&config)?;
        }
    };

    Ok(())
}
