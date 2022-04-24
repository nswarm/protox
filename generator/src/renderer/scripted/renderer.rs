use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use rhai::module_resolvers::FileModuleResolver;
use rhai::{Dynamic, Engine, Scope, ScriptFnMetadata, AST};

use crate::renderer::context::{FileContext, MetadataContext};
use crate::renderer::scripted::api;
use crate::renderer::scripted::api::output::Output;
use crate::renderer::{Renderer, RendererConfig};
use crate::{DisplayNormalized, CONFIG_FILE_NAME};

pub const SCRIPT_EXT: &'static str = "rhai";
pub const MAIN_SCRIPT_NAME: &'static str = "main";
pub const RENDER_FILE_FN_NAME: &'static str = "render_file";
pub const RENDER_METADATA_FN_NAME: &'static str = "render_metadata";

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

    fn create_engine() -> Engine {
        let mut engine = Engine::new();
        // todo go through & set options
        engine.on_print(|msg| info!("[script] {}", msg));
        engine.on_debug(|msg, _, pos| debug!("[script] {}: {}", pos, msg));
        engine.set_max_expr_depths(128, 64);
        engine.set_max_operations(0);
        api::register(&mut engine);
        engine
    }

    fn main_ast_or_error(&self) -> Result<&AST> {
        match &self.main_ast {
            None => Err(anyhow!("`{}` script file is not loaded.", MAIN_SCRIPT_NAME)),
            Some(ast) => Ok(ast),
        }
    }

    fn render<W: Write>(
        &self,
        context: rhai::Dynamic,
        fn_name: &str,
        writer: &mut W,
    ) -> Result<()> {
        let mut scope = Scope::new();
        let ast = self.main_ast_or_error()?;
        let output = Output::with_config(self.config.scripted.clone());
        let result: Output = self
            .engine
            .call_fn(&mut scope, ast, fn_name, (context, output))
            .with_context(|| format!("Error returned from script function: {}'", fn_name))?;
        writer.write(result.to_string().as_bytes())?;
        Ok(())
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
        self.config = self.load_config(&input_root.join(CONFIG_FILE_NAME))?;
        let resolver = FileModuleResolver::new_with_path_and_extension(input_root, SCRIPT_EXT);
        self.engine.set_module_resolver(resolver);
        self.main_ast = Some(compile_file(
            &mut self.engine,
            &main_script_path(input_root),
        )?);
        Ok(())
    }

    fn reset(&mut self) {
        self.main_ast = None;
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn has_metadata(&self) -> bool {
        if let Some(ast) = &self.main_ast {
            return ast
                .iter_functions()
                .find(|f: &ScriptFnMetadata| f.name == RENDER_METADATA_FN_NAME)
                .is_some();
        }
        false
    }

    fn render_metadata<W: Write>(&self, context: MetadataContext, writer: &mut W) -> Result<()> {
        self.render(Dynamic::from(context), RENDER_METADATA_FN_NAME, writer)
    }

    fn render_file<W: Write>(&self, context: FileContext, writer: &mut W) -> Result<()> {
        self.render(Dynamic::from(context), RENDER_FILE_FN_NAME, writer)
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

#[cfg(test)]
mod tests {
    use crate::renderer::context::{FileContext, MetadataContext};
    use anyhow::Result;
    use prost_types::FileDescriptorProto;
    use std::path::PathBuf;

    use crate::renderer::scripted::renderer::ScriptedRenderer;
    use crate::renderer::{Renderer, RendererConfig};

    #[test]
    fn render_file() -> Result<()> {
        let expected = "FileName".to_owned();
        let file = &FileDescriptorProto {
            name: Some(expected.clone()),
            ..Default::default()
        };
        let context = FileContext::new(file, &RendererConfig::default())?;
        let mut renderer = ScriptedRenderer::new();
        renderer.load_test_script(
            r#"fn render_file(f, o) {
                o.append(`hello ${f.source_file}!`);
                o
            }"#,
        )?;

        let mut output = Vec::new();
        renderer.render_file(context, &mut output)?;
        assert_eq!(String::from_utf8(output)?, format!("hello {}!", expected));
        Ok(())
    }

    #[test]
    fn render_metadata() -> Result<()> {
        let expected = "some/directory";
        let context = MetadataContext::with_relative_dir(&PathBuf::from(expected))?;
        let mut renderer = ScriptedRenderer::new();
        renderer.load_test_script(
            r#"fn render_metadata(m, o) {
                o.append(`hello ${m.directory}!`);
                o
            }"#,
        )?;

        let mut output = Vec::new();
        renderer.render_metadata(context, &mut output)?;
        assert_eq!(String::from_utf8(output)?, format!("hello {}!", expected));
        Ok(())
    }

    #[test]
    fn has_metadata() -> Result<()> {
        let mut renderer = ScriptedRenderer::new();
        assert!(!renderer.has_metadata());
        renderer.load_test_script("fn some_fn() {}")?;
        assert!(!renderer.has_metadata());
        renderer.load_test_script("fn render_metadata(m, o) {}")?;
        assert!(renderer.has_metadata());
        Ok(())
    }
}
