use anyhow::Result;

use crate::in_out_generator::InOutGenerator;
use crate::renderer::scripted::renderer::ScriptedRenderer;
use crate::{Config, InOutConfig};

mod renderer;

pub fn generate(config: &Config) -> Result<()> {
    Generator {
        config,
        renderer: ScriptedRenderer::new(),
    }
    .generate()
}

struct Generator<'a> {
    config: &'a Config,
    renderer: ScriptedRenderer,
}
impl<'a> InOutGenerator<ScriptedRenderer> for Generator<'a> {
    fn name(&self) -> &str {
        "Scripts"
    }

    fn renderer(&mut self) -> &mut ScriptedRenderer {
        &mut self.renderer
    }

    fn app_config(&self) -> &Config {
        &self.config
    }

    fn in_out_configs(&self) -> Vec<InOutConfig> {
        self.app_config().scripts.clone()
    }
}
