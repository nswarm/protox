use std::io::Write;
use std::iter::once;
use std::path::{Path, PathBuf};

use crate::DisplayNormalized;
use anyhow::{anyhow, Context, Result};
use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorProto, FileDescriptorSet};
use rhai::module_resolvers::FileModuleResolver;
use rhai::{Dynamic, Engine, EvalAltResult, FuncArgs, Module, Scope, AST};
use walkdir::WalkDir;

use crate::renderer::context::{
    FieldContext, FileContext, ImportContext, MessageContext, MetadataContext,
};
use crate::renderer::scripted::api;
use crate::renderer::scripted::api::output::Output;
use crate::renderer::{Renderer, RendererConfig};

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

    fn load_script(&mut self, script: &str) -> Result<()> {
        self.main_ast = Some(self.engine.compile(script)?);
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
        let mut output = Output::new();
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
        writer.write(result.to_string().as_bytes())?;
        Ok(())
    }
}

fn main_script_path(root: &Path) -> PathBuf {
    root.join(MAIN_SCRIPT_NAME).with_extension(SCRIPT_EXT)
}

fn file_name(file: &mut FileDescriptorProto) -> String {
    file.name
        .as_ref()
        .map(|s| s.clone())
        .unwrap_or("".to_string())
}

fn compile_file(engine: &mut rhai::Engine, path: &Path) -> Result<AST> {
    engine
        .compile_file(path.to_path_buf())
        .with_context(|| format!("Error compiling script: {}", path.display_normalized()))
}

#[cfg(test)]
mod tests {
    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::renderer::ScriptedRenderer;
    use crate::renderer::{Renderer, RendererConfig};
    use anyhow::Result;
    use prost_types::FileDescriptorProto;
    use std::{fs, io};
    use tempfile::tempdir;

    #[test]
    fn render_file_test() -> Result<()> {
        let mut renderer = ScriptedRenderer::new();
        renderer.load_script(
            r#"fn render_file(context, output) { output.append(context.source_file); output }"#,
        )?;
        let mut buffer = Vec::new();
        let context = FileContext::new(&file_proto(), &RendererConfig::default())?;
        renderer.render_file(&context, &mut buffer)?;
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            context.source_file().to_string() + "\n"
        );
        Ok(())
    }

    fn file_proto() -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some("some name".to_string()),
            package: Some("some.package".to_string()),
            ..Default::default()
        }
    }
}
