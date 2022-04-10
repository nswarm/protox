use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rhai::module_resolvers::FileModuleResolver;
use rhai::{Dynamic, Engine, Scope, AST};

use crate::renderer::context::{FileContext, MetadataContext};
use crate::renderer::scripted::api;
use crate::renderer::scripted::api::output::Output;
use crate::renderer::{Renderer, RendererConfig};
use crate::DisplayNormalized;

pub const SCRIPT_EXT: &'static str = "rhai";
pub const MAIN_SCRIPT_NAME: &'static str = "main";
pub const RENDER_FILE_FN_NAME: &'static str = "render_file";

pub struct ScriptedRenderer {
    engine: Engine,
    main_ast: Option<AST>,
    config: RendererConfig,
}

impl ScriptedRenderer {
    pub fn new() -> Self {
        Self {
            engine: Self::create_engine(),
            main_ast: None,
            config: RendererConfig::default(),
        }
    }

    pub fn with_config(config: RendererConfig) -> Self {
        Self {
            engine: Self::create_engine(),
            main_ast: None,
            config,
        }
    }

    fn create_engine() -> Engine {
        let mut engine = Engine::new();
        // todo go through & set options
        api::register(&mut engine);
        engine
    }

    fn main_ast_or_error(&self) -> Result<&AST> {
        match &self.main_ast {
            None => Err(anyhow!("`{}` script file is not loaded.", MAIN_SCRIPT_NAME)),
            Some(ast) => Ok(ast),
        }
    }

    #[cfg(test)]
    pub fn load_test_script(&mut self, script: &str) -> Result<()> {
        self.main_ast = Some(
            self.engine
                .compile(script)
                .with_context(|| format!("Error compiling script:\n{}", script))?,
        );
        Ok(())
    }
}

impl Renderer for ScriptedRenderer {
    fn load(&mut self, input_root: &Path) -> anyhow::Result<()> {
        let resolver = FileModuleResolver::new_with_path_and_extension(input_root, SCRIPT_EXT);
        self.engine.set_module_resolver(resolver);
        self.main_ast = Some(compile_file(
            &mut self.engine,
            &main_script_path(input_root),
        )?);
        Ok(())
    }

    fn reset(&mut self) {
        // todo
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn has_metadata(&self) -> bool {
        // todo
        false
    }

    fn render_metadata<W: Write>(&self, context: MetadataContext, writer: &mut W) -> Result<()> {
        todo!()
    }

    fn render_file<W: Write>(&self, context: &FileContext, writer: &mut W) -> Result<()> {
        let mut scope = Scope::new();
        let ast = self.main_ast_or_error()?;
        let output = Output::new();
        let result: Output = self
            .engine
            .call_fn(
                &mut scope,
                ast,
                RENDER_FILE_FN_NAME,
                (Dynamic::from(context.clone()), output),
            )
            .with_context(|| {
                format!(
                    "Error returned from script function: {}'",
                    RENDER_FILE_FN_NAME
                )
            })?;
        writer.write(result.to_owned().as_bytes())?;
        Ok(())
    }
}

fn main_script_path(root: &Path) -> PathBuf {
    root.join(MAIN_SCRIPT_NAME).with_extension(SCRIPT_EXT)
}

fn compile_file(engine: &mut rhai::Engine, path: &Path) -> Result<AST> {
    engine
        .compile_file(path.to_path_buf())
        .with_context(|| format!("Error compiling script: {}", path.display_normalized()))
}
