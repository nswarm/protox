use anyhow::Result;

use renderer::Renderer;

use crate::in_out_generator::InOutGenerator;
use crate::{Config, InOutConfig};

mod helper;
mod renderer;

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
