use anyhow::Result;

use renderer::Renderer;

use crate::in_out_generator::InOutGenerator;
use crate::{Config, InOutConfig};

mod renderer;

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
