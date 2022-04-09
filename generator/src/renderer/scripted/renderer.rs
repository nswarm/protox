use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
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

    fn load_script(&mut self, script: &str) -> Result<()> {
        self.main_ast = Some(
            self.engine
                .compile(script)
                // todo insert line numbers after \n?
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

fn file_name(file: &mut FileDescriptorProto) -> String {
    file.name
        .as_ref()
        .map(|s| s.clone())
        .unwrap_or("".to_owned())
}

fn compile_file(engine: &mut rhai::Engine, path: &Path) -> Result<AST> {
    engine
        .compile_file(path.to_path_buf())
        .with_context(|| format!("Error compiling script: {}", path.display_normalized()))
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use anyhow::Result;
    use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorProto, FileOptions};
    use tempfile::tempdir;

    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::renderer::ScriptedRenderer;
    use crate::renderer::{Renderer, RendererConfig};

    mod file_context {
        use anyhow::Result;
        use prost_types::FileDescriptorProto;

        use crate::renderer::context::FileContext;
        use crate::renderer::scripted::renderer::tests::{
            default_enum_proto, default_file_proto, default_message_proto, file_with_enums,
            file_with_imports, file_with_messages, test_render_file,
        };
        use crate::renderer::RendererConfig;

        #[test]
        fn source_file() -> Result<()> {
            let proto = default_file_proto();
            let context = FileContext::new(&proto, &RendererConfig::default())?;
            test_render_file(
                &context,
                "output.append(context.source_file);",
                context.source_file(),
            )
        }

        // The rest are tested in their own section since we use render_file anyway.
    }

    mod import_context {
        use crate::renderer::context::{FileContext, ImportContext};
        use anyhow::Result;
        use prost_types::FileDescriptorProto;

        use crate::renderer::scripted::renderer::tests::{
            default_file_proto, file_with_imports, test_render_file,
        };
        use crate::renderer::RendererConfig;

        #[test]
        fn file_path() -> Result<()> {
            run_test("file_path", "relative/path/file.txt")
        }

        #[test]
        fn file_name() -> Result<()> {
            run_test("file_name", "file")
        }

        #[test]
        fn file_name_with_ext() -> Result<()> {
            run_test("file_name_with_ext", "file.txt")
        }

        fn run_test(method: &str, expected_output: &str) -> Result<()> {
            let proto = default_file_proto();
            let context = file_with_imports(&["relative/path/file.txt"])?;
            test_render_file(
                &context,
                &format!("output.append(context.imports[0].{});", method),
                expected_output,
            )
        }
    }

    mod enum_context {}

    mod message_context {}

    mod field_context {}

    mod metadata_context {}

    mod file_options {
        use crate::renderer::scripted::renderer::tests::{
            default_file_proto, file_with_options, test_render_file,
        };
        use anyhow::Result;
        use prost::Extendable;
        use prost_types::FileOptions;

        macro_rules! opt_test {
            ($name: ident, $value: expr) => {
                #[test]
                fn $name() -> Result<()> {
                    let options = FileOptions {
                        $name: Some($value),
                        ..Default::default()
                    };
                    run_test(options, stringify!($name), &$value.to_string())
                }
            };
        }

        opt_test!(deprecated, true);
        opt_test!(go_package, "some value".to_owned());
        opt_test!(java_package, "some value".to_owned());
        opt_test!(ruby_package, "some value".to_owned());
        opt_test!(csharp_namespace, "some value".to_owned());
        opt_test!(php_namespace, "some value".to_owned());
        opt_test!(php_metadata_namespace, "some value".to_owned());
        opt_test!(swift_prefix, "some value".to_owned());
        opt_test!(java_generic_services, true);
        opt_test!(java_outer_classname, "some value".to_owned());
        opt_test!(java_multiple_files, true);
        opt_test!(cc_generic_services, true);
        opt_test!(cc_enable_arenas, true);
        opt_test!(java_string_check_utf8, true);
        opt_test!(optimize_for, 123);
        opt_test!(php_generic_services, true);
        opt_test!(php_class_prefix, "some value".to_owned());
        opt_test!(py_generic_services, true);
        opt_test!(objc_class_prefix, "some value".to_owned());

        fn run_test(options: FileOptions, method: &str, expected_output: &str) -> Result<()> {
            let context = file_with_options(options)?;
            test_render_file(
                &context,
                &format!("output.append(context.options.{});", method),
                expected_output,
            )
        }

        #[test]
        fn kv_option() -> Result<()> {
            let mut options = FileOptions::default();
            options.set_extension_data(
                proto_options::FILE_KEY_VALUE,
                vec!["test_key=some_value".to_owned()],
            );
            let context = file_with_options(options)?;
            test_render_file(
                &context,
                "output.append(context.options[\"test_key\"]);",
                "some_value",
            )
        }
    }

    fn default_file_proto() -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some("name".to_owned()),
            ..Default::default()
        }
    }

    fn default_message_proto(name: &str) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.to_owned()),
            ..Default::default()
        }
    }

    fn default_enum_proto(name: &str) -> EnumDescriptorProto {
        EnumDescriptorProto {
            name: Some(name.to_owned()),
            ..Default::default()
        }
    }

    fn file_with_imports(imports: &[&str]) -> Result<FileContext> {
        let mut proto = default_file_proto();
        for import in imports {
            proto.dependency.push(import.to_string());
        }
        FileContext::new(&proto, &RendererConfig::default())
    }

    fn file_with_enums(enums: Vec<EnumDescriptorProto>) -> Result<FileContext> {
        let mut proto = default_file_proto();
        proto.enum_type = enums;
        FileContext::new(&proto, &RendererConfig::default())
    }

    fn file_with_messages(messages: Vec<DescriptorProto>) -> Result<FileContext> {
        let mut proto = default_file_proto();
        proto.message_type = messages;
        FileContext::new(&proto, &RendererConfig::default())
    }

    fn file_with_options(options: FileOptions) -> Result<FileContext> {
        let mut proto = default_file_proto();
        proto.options = Some(options);
        FileContext::new(&proto, &RendererConfig::default())
    }

    fn test_render_file(
        context: &FileContext,
        script_content: &str,
        expected_output: &str,
    ) -> Result<()> {
        let mut renderer = ScriptedRenderer::new();
        renderer.load_script(&format!(
            r#"
            fn render_file(context, output) {{
                {}
                output
            }}"#,
            script_content
        ))?;
        let mut buffer = Vec::new();
        renderer.render_file(&context, &mut buffer)?;
        assert_eq!(String::from_utf8(buffer).unwrap(), expected_output);
        Ok(())
    }
}
