use anyhow::Result;

use crate::in_out_generator::InOutGenerator;
use crate::renderer::scripted::renderer::ScriptedRenderer;
use crate::{Config, InOutConfig};

mod api;
mod renderer;

#[cfg(test)]
mod integration_tests;

pub const SCRIPT_EXT: &'static str = "rhai";
pub const MAIN_SCRIPT_NAME: &'static str = "main";
pub const RENDER_FILE_FN_NAME: &'static str = "render_file";
pub const RENDER_METADATA_FN_NAME: &'static str = "render_metadata";

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
