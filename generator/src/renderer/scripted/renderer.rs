use crate::renderer::context::{FileContext, MetadataContext};
use crate::renderer::{Renderer, RendererConfig};
use anyhow::{anyhow, Result};
use prost_types::{DescriptorProto, FileDescriptorProto, FileDescriptorSet};
use rhai::module_resolvers::FileModuleResolver;
use rhai::{Dynamic, Engine, FuncArgs, Module, Scope, AST};
use std::io::Write;
use std::iter::once;
use std::path::Path;
use walkdir::WalkDir;

pub const SCRIPT_EXT: &'static str = "rhai";
pub const MAIN_SCRIPT_NAME: &'static str = "main";
pub const MAIN_FN_NAME: &'static str = "main";

pub struct ScriptedRenderer {
    engine: Engine,
    main_ast: Option<AST>,
}

impl ScriptedRenderer {
    pub fn new() -> Self {
        Self {
            engine: Self::create_engine(),
            main_ast: None,
        }
    }

    fn create_engine() -> Engine {
        let mut engine = Engine::new();
        engine.register_type::<FileDescriptorSet>();
        engine
            .register_type::<FileDescriptorProto>()
            .register_get("name", file_name);
        engine.register_type::<DescriptorProto>();
        // todo set options
        // todo register types
        // todo register fns
        engine
    }
}

impl Renderer for ScriptedRenderer {
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

    fn config(&self) -> &RendererConfig {
        todo!()
    }

    fn has_metadata(&self) -> bool {
        todo!()
    }

    fn render_metadata<W: Write>(&self, context: MetadataContext, writer: &mut W) -> Result<()> {
        todo!()
    }

    fn render_file<W: Write>(&self, context: &FileContext, writer: &mut W) -> Result<()> {
        todo!()
    }

    // fn render(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> anyhow::Result<()> {
    //     let ast = match &self.main_ast {
    //         None => {
    //             return Err(anyhow!(
    //                 "Could not find entry point '{}.{}'.",
    //                 MAIN_SCRIPT_NAME,
    //                 SCRIPT_EXT
    //             ))
    //         }
    //         Some(ast) => ast,
    //     };
    //     // todo parse context + pass into main
    //     let mut scope = Scope::new();
    //     let files: rhai::Array = vec![
    //         Dynamic::from(FileDescriptorProto {
    //             name: Some("hihihi".to_string()),
    //             ..Default::default()
    //         }),
    //         Dynamic::from(FileDescriptorProto {
    //             name: Some("hellohello".to_string()),
    //             ..Default::default()
    //         }),
    //     ];
    //     let result: i64 = self
    //         .engine
    //         .call_fn(&mut scope, ast, MAIN_FN_NAME, (files,))?;
    //     return match result {
    //         0 => Ok(()),
    //         _ => Err(anyhow!("Script exited with error code {}", result)),
    //     };
    // }
}

fn file_name(file: &mut FileDescriptorProto) -> String {
    file.name
        .as_ref()
        .map(|s| s.clone())
        .unwrap_or("".to_string())
}

// struct File {
//     proto: FileDescriptorProto,
// }
// impl File {}
