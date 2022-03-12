mod case;
mod context;
mod helper;
mod option_key_value;
mod primitive;
mod proto;
mod renderer;
mod renderer_config;

use crate::in_out_generator::InOutGenerator;
use crate::template_renderer::renderer::Renderer;
use crate::{Config, InOutConfig};
use anyhow::Result;
pub use renderer_config::RendererConfig;

pub const CONFIG_FILE_NAME: &'static str = "config.json";
pub const TEMPLATE_EXT: &'static str = "hbs";
pub const METADATA_TEMPLATE_NAME: &'static str = "metadata";
pub const FILE_TEMPLATE_NAME: &'static str = "file";

pub fn generate(config: &Config) -> Result<()> {
    Generator {
        config,
        renderer: Renderer::new(),
    }
    .generate()
}

struct Generator<'a> {
    config: &'a Config,
    renderer: Renderer<'a>,
}
impl<'a> InOutGenerator<Renderer<'a>> for Generator<'a> {
    fn name(&self) -> &str {
        "Templates"
    }

    fn renderer(&mut self) -> &mut Renderer<'a> {
        &mut self.renderer
    }

    fn app_config(&self) -> &Config {
        &self.config
    }

    fn in_out_configs(&self) -> Vec<InOutConfig> {
        self.app_config().templates.clone()
    }
}
