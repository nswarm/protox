mod renderer;

use crate::in_out_generator::InOutGenerator;
use crate::scripted_renderer::renderer::Renderer;
use crate::{Config, InOutConfig};
use anyhow::Result;

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
    renderer: Renderer,
}
impl<'a> InOutGenerator<Renderer> for Generator<'a> {
    fn name(&self) -> &str {
        "Scripts"
    }

    fn renderer(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    fn app_config(&self) -> &Config {
        &self.config
    }

    fn in_out_configs(&self) -> Vec<InOutConfig> {
        self.app_config().scripts.clone()
    }
}
