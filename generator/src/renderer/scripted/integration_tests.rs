use anyhow::Result;
use prost_types::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FileDescriptorProto,
    FileOptions,
};

use crate::renderer::context::FileContext;
use crate::renderer::scripted::renderer::ScriptedRenderer;
use crate::renderer::{Renderer, RendererConfig};

mod file_context {
    use anyhow::Result;

    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::integration_tests::{default_file_proto, test_script};
    use crate::renderer::RendererConfig;

    #[test]
    fn source_file() -> Result<()> {
        let proto = default_file_proto();
        let context = FileContext::new(&proto, &RendererConfig::default())?;
        test_script(
            &context,
            "output.append(context.source_file);",
            context.source_file(),
        )
    }

    // The rest are tested in their own section since we go through FileContext anyway.
}

mod import_context {
    use anyhow::Result;

    use crate::renderer::scripted::integration_tests::{
        default_file_proto, file_with_imports, test_script,
    };

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
        let context = file_with_imports(&["relative/path/file.txt"])?;
        test_script(
            &context,
            &format!("output.append(context.imports[0].{});", method),
            expected_output,
        )
    }
}

mod enum_context {
    use anyhow::Result;
    use prost_types::{EnumDescriptorProto, EnumValueDescriptorProto};

    use crate::renderer::scripted::integration_tests::{enum_proto, file_with_enums, test_script};

    #[test]
    fn name() -> Result<()> {
        run_test("name", enum_proto().name())
    }

    // Others are tested in their own sections.

    fn run_test(method: &str, expected_output: &str) -> Result<()> {
        let context = file_with_enums(vec![enum_proto()])?;
        test_script(
            &context,
            &format!("output.append(context.enums[0].{});", method),
            expected_output,
        )
    }
}

mod enum_value_context {
    use anyhow::Result;
    use prost_types::{EnumDescriptorProto, EnumValueDescriptorProto};

    use crate::renderer::scripted::integration_tests::{enum_proto, file_with_enums, test_script};

    #[test]
    fn name() -> Result<()> {
        run_test("name", enum_proto().value[0].name())
    }

    #[test]
    fn number() -> Result<()> {
        run_test("number", &enum_proto().value[0].number().to_string())
    }

    fn run_test(method: &str, expected_output: &str) -> Result<()> {
        let context = file_with_enums(vec![enum_proto()])?;
        test_script(
            &context,
            &format!(
                r#"
            let enum = context.enums[0];
            let enum_value = enum.values[0];
            output.append(enum_value.{});
            "#,
                method
            ),
            expected_output,
        )
    }
}

mod message_context {
    use crate::renderer::scripted::integration_tests::{
        default_message_proto, file_with_messages, test_script,
    };
    use anyhow::Result;

    #[test]
    fn name() -> Result<()> {
        run_test("name", "SomeMessage")
    }

    // Others are tested in their own sections.

    fn run_test(method: &str, expected_output: &str) -> Result<()> {
        let message = default_message_proto("SomeMessage");
        let context = file_with_messages(vec![message])?;
        test_script(
            &context,
            &format!("output.append(context.messages[0].{});", method),
            expected_output,
        )
    }
}

mod field_context {}

mod metadata_context {}

mod file_options {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::FileOptions;

    use crate::renderer::scripted::integration_tests::{file_with_options, test_script};

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
        test_script(
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
        )?;
        let context = file_with_options(options)?;
        test_script(
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

fn enum_proto() -> EnumDescriptorProto {
    EnumDescriptorProto {
        name: Some("EnumName".to_owned()),
        value: vec![EnumValueDescriptorProto {
            name: Some("EnumValueName".to_owned()),
            number: Some(123),
            ..Default::default()
        }],
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

fn test_script(context: &FileContext, script_content: &str, expected_output: &str) -> Result<()> {
    let mut renderer = ScriptedRenderer::new();
    renderer.load_test_script(&format!(
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
