use crate::render::Render;
use anyhow::{anyhow, Result};
use prost_types::FileDescriptorSet;
use rhai::module_resolvers::FileModuleResolver;
use rhai::{Engine, Module, Scope, AST};
use std::path::Path;
use walkdir::WalkDir;

pub const SCRIPT_EXT: &'static str = "rhai";
pub const MAIN_SCRIPT_NAME: &'static str = "main";
pub const MAIN_FN_NAME: &'static str = "main";

pub struct Renderer {
    engine: Engine,
    main_ast: Option<AST>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            engine: Self::create_engine(),
            main_ast: None,
        }
    }

    fn create_engine() -> Engine {
        let engine = Engine::new();
        // todo set options
        // todo register types
        // todo register fns
        engine
    }
}

impl Render for Renderer {
    fn load(&mut self, input_root: &Path) -> anyhow::Result<()> {
        let resolver = FileModuleResolver::new_with_path_and_extension(input_root, SCRIPT_EXT);
        self.engine.set_module_resolver(resolver);
        self.main_ast = Some(
            self.engine
                .compile_file(input_root.join(MAIN_SCRIPT_NAME).with_extension(SCRIPT_EXT))?,
        );
        Ok(())
    }

    fn reset(&mut self) {
        todo!()
    }

    fn render(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> anyhow::Result<()> {
        let ast = match &self.main_ast {
            None => {
                return Err(anyhow!(
                    "Could not find entry point '{}.{}'.",
                    MAIN_SCRIPT_NAME,
                    SCRIPT_EXT
                ))
            }
            Some(ast) => ast,
        };
        // todo parse context + pass into main
        let mut scope = Scope::new();
        let result: i64 = self.engine.call_fn(&mut scope, ast, MAIN_FN_NAME, ())?;
        return match result {
            0 => Ok(()),
            _ => Err(anyhow!("Script exited with error code {}", result)),
        };
    }
}
