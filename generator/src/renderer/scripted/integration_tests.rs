use anyhow::Result;
use prost_types::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    FileDescriptorProto, FileOptions,
};

use crate::renderer::context::{FileContext, MetadataContext};
use crate::renderer::scripted::renderer::ScriptedRenderer;
use crate::renderer::{Renderer, RendererConfig};

mod file_context {
    use anyhow::Result;

    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::integration_tests::{default_file_proto, test_file_script};
    use crate::renderer::RendererConfig;

    #[test]
    fn source_file() -> Result<()> {
        let proto = default_file_proto();
        let context = FileContext::new(&proto, &RendererConfig::default())?;
        let expected = context.source_file().to_owned();
        test_file_script(context, "output.append(context.source_file);", &expected)
    }

    // Others accessors are tested in their own sections.
}

mod import_context {
    use anyhow::Result;

    use crate::renderer::scripted::integration_tests::{file_with_imports, test_file_script};

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
        test_file_script(
            context,
            &format!("output.append(context.imports[0].{});", method),
            expected_output,
        )
    }
}

mod enum_context {
    use anyhow::Result;

    use crate::renderer::scripted::integration_tests::{
        enum_proto, file_with_enums, test_file_script,
    };

    #[test]
    fn name() -> Result<()> {
        run_test("name", enum_proto().name())
    }

    // Others accessors are tested in their own sections.

    fn run_test(method: &str, expected_output: &str) -> Result<()> {
        let context = file_with_enums(vec![enum_proto()])?;
        test_file_script(
            context,
            &format!("output.append(context.enums[0].{});", method),
            expected_output,
        )
    }
}

mod enum_value_context {
    use anyhow::Result;

    use crate::renderer::scripted::integration_tests::{
        enum_proto, file_with_enums, test_file_script,
    };

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
        test_file_script(
            context,
            &format!(
                r#"
            let enum = context.enums[0];
            let enum_value = enum.values[0];
            output.append(enum_value.{}.to_string());
            "#,
                method
            ),
            expected_output,
        )
    }
}

mod message_context {
    use anyhow::Result;

    use crate::renderer::scripted::integration_tests::{
        default_message_proto, file_with_messages, test_file_script,
    };

    #[test]
    fn name() -> Result<()> {
        run_test("name", "SomeMessage")
    }

    // Others accessors are tested in their own sections.

    fn run_test(method: &str, expected_output: &str) -> Result<()> {
        let message = default_message_proto("SomeMessage");
        let context = file_with_messages(vec![message])?;
        test_file_script(
            context,
            &format!("output.append(context.messages[0].{});", method),
            expected_output,
        )
    }
}

mod field_context {
    use anyhow::Result;
    use prost_types::field_descriptor_proto::{Label, Type};
    use prost_types::{DescriptorProto, FieldDescriptorProto, MessageOptions};

    use crate::renderer::scripted::integration_tests::{
        default_message_proto, file_with_messages, test_file_script,
    };

    #[test]
    fn name() -> Result<()> {
        run_test(field(), "name", "some_field")
    }
    #[test]
    fn fully_qualified_type() -> Result<()> {
        run_test(field(), "fully_qualified_type", "package.SomeType")
    }
    #[test]
    fn relative_type() -> Result<()> {
        run_test(field(), "relative_type", "package.SomeType")
    }
    #[test]
    fn is_oneof() -> Result<()> {
        run_test(field(), "is_oneof", "true")
    }

    #[test]
    fn is_array() -> Result<()> {
        run_test(array_field(), "is_array", "true")
    }

    #[test]
    fn is_map() -> Result<()> {
        run_map_test("is_map", "true")
    }
    #[test]
    fn fully_qualified_key_type() -> Result<()> {
        run_map_test("fully_qualified_key_type", "string")
    }
    #[test]
    fn fully_qualified_value_type() -> Result<()> {
        run_map_test("fully_qualified_value_type", "int32")
    }
    #[test]
    fn relative_key_type() -> Result<()> {
        run_map_test("relative_key_type", "string")
    }
    #[test]
    fn relative_value_type() -> Result<()> {
        run_map_test("relative_value_type", "int32")
    }

    fn field() -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some("some_field".to_owned()),
            type_name: Some(".package.SomeType".to_owned()),
            oneof_index: Some(0),
            ..Default::default()
        }
    }

    fn array_field() -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some("some_field".to_owned()),
            type_name: Some(".package.SomeType".to_owned()),
            label: Some(Label::Repeated as i32),
            ..Default::default()
        }
    }

    fn map_field() -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some("some_field".to_owned()),
            type_name: Some(".MapOwner.MapEntry".to_owned()),
            ..Default::default()
        }
    }

    fn run_test(field: FieldDescriptorProto, method: &str, expected_output: &str) -> Result<()> {
        let context = file_with_messages(vec![message(vec![field])])?;
        test_file_script(
            context,
            &format!(
                r#"
            let message = context.messages[0];
            let field = message.fields[0];
            output.append(field.{}.to_string());
            "#,
                method
            ),
            expected_output,
        )
    }

    fn run_map_test(method: &str, expected_output: &str) -> Result<()> {
        let context = file_with_messages(vec![map_message()])?;
        test_file_script(
            context,
            &format!(
                r#"
            let message = context.messages[0];
            let field = message.fields[0];
            output.append(field.{}.to_string());
            "#,
                method
            ),
            expected_output,
        )
    }

    fn message(fields: Vec<FieldDescriptorProto>) -> DescriptorProto {
        let mut message = default_message_proto("SomeName");
        message.field = fields;
        message
    }

    fn map_message() -> DescriptorProto {
        let mut message = default_message_proto("MapOwner");
        let mut map_entry = DescriptorProto {
            name: Some("MapEntry".to_owned()),
            ..Default::default()
        };
        map_entry.options = Some(MessageOptions {
            map_entry: Some(true),
            ..Default::default()
        });
        map_entry.field.push(FieldDescriptorProto {
            name: Some("key".to_owned()),
            r#type: Some(Type::String as i32),
            ..Default::default()
        });
        map_entry.field.push(FieldDescriptorProto {
            name: Some("value".to_owned()),
            r#type: Some(Type::Int32 as i32),
            ..Default::default()
        });
        message.nested_type.push(map_entry);
        message.field.push(map_field());
        message
    }
}

mod metadata_context {
    use crate::renderer::context::MetadataContext;
    use crate::renderer::{Renderer, RendererConfig};
    use anyhow::Result;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::renderer::scripted::integration_tests::{
        default_message_proto, file_with_messages, test_file_script, test_metadata_script,
    };
    use crate::renderer::scripted::renderer::ScriptedRenderer;

    #[test]
    fn directory() -> Result<()> {
        let context = MetadataContext::with_relative_dir(&PathBuf::from("some/dir/path"))?;
        test_metadata_script(
            context,
            r#"output.append(context.directory.to_string());"#,
            "some/dir/path",
        )
    }

    #[test]
    fn directory_path() -> Result<()> {
        let context = MetadataContext::with_relative_dir(&PathBuf::from("some/dir/path"))?;
        test_metadata_script(
            context,
            r#"output.append(context.directory.to_string());"#,
            "some/dir/path",
        )
    }

    #[test]
    fn file_names() -> Result<()> {
        let mut context = MetadataContext::new();
        context.push_file(&PathBuf::from("file0.ext"));
        context.push_file(&PathBuf::from("file1.ext"));
        test_metadata_script(
            context,
            r#"
            for file_name in context.file_names {
                output.append(file_name);
            }
            "#,
            "file0file1",
        )
    }

    #[test]
    fn file_names_with_ext() -> Result<()> {
        let mut context = MetadataContext::new();
        context.push_file(&PathBuf::from("file0.ext"));
        context.push_file(&PathBuf::from("file1.ext"));
        test_metadata_script(
            context,
            r#"
            for file_name in context.file_names_with_ext {
                output.append(file_name);
            }
            "#,
            "file0.extfile1.ext",
        )
    }

    #[test]
    fn subdirectories() -> Result<()> {
        let mut context = MetadataContext::new();
        context.push_subdirectory(&PathBuf::from("subdir0"));
        context.push_subdirectory(&PathBuf::from("subdir1"));
        test_metadata_script(
            context,
            r#"
            for subdir in context.subdirectories {
                output.append(subdir);
            }
            "#,
            "subdir0subdir1",
        )
    }

    #[test]
    fn package_files_full() -> Result<()> {
        let mut context = MetadataContext::new();
        let mut package_files = HashMap::<String, PathBuf>::new();
        package_files.insert(
            "some.package.0".to_owned(),
            PathBuf::from("some_file_0.ext"),
        );
        package_files.insert(
            "some.package.1".to_owned(),
            PathBuf::from("some_file_1.ext"),
        );
        context.append_package_files(package_files);
        test_metadata_script(
            context,
            r#"
            for pf in context.package_files_full {
                output.append(pf.file_package);
                output.append(pf.file_name);
            }
            "#,
            "some.package.0some_file_0.extsome.package.1some_file_1.ext",
        )
    }

    #[test]
    fn package_file_tree() -> Result<()> {
        let mut context = MetadataContext::new();
        let mut package_files = HashMap::<String, PathBuf>::new();
        package_files.insert("0.1.2".to_owned(), PathBuf::from("file0"));
        package_files.insert("0.1".to_owned(), PathBuf::from("file1"));
        package_files.insert("0.3".to_owned(), PathBuf::from("file2"));
        context.append_package_files(package_files);
        let mut renderer = ScriptedRenderer::new();
        renderer.load_test_script(
            r#"
            fn print_children(children) {
                let keys = children.keys();
                keys.sort();
                for key in keys {
                    output.append(`[${key}]`);
                }
                let values = children.values();
                values.sort();
                for node in values {
                    output.append(node.file_name);
                    print_children!(node.children);
                }
            }
            fn render_metadata(context, output) {
                print_children!(context.package_file_tree);
                output
            }
            "#,
        )?;
        let mut buffer = Vec::new();
        renderer.render_metadata(context, &mut buffer)?;
        assert_eq!(String::from_utf8(buffer)?, "[0][1][3]file1[2]file0file2");
        Ok(())
    }
}

macro_rules! opt_test {
    ($opt_type: ident, $name: ident, $value: expr) => {
        #[test]
        fn $name() -> Result<()> {
            let options = $opt_type {
                $name: Some($value),
                ..Default::default()
            };
            run_test(options, stringify!($name), &$value.to_string())
        }
    };
}

mod file_options {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::FileOptions;

    use crate::renderer::scripted::integration_tests::{file_with_options, test_file_script};

    opt_test!(FileOptions, deprecated, true);
    opt_test!(FileOptions, go_package, "some value".to_owned());
    opt_test!(FileOptions, java_package, "some value".to_owned());
    opt_test!(FileOptions, ruby_package, "some value".to_owned());
    opt_test!(FileOptions, csharp_namespace, "some value".to_owned());
    opt_test!(FileOptions, php_namespace, "some value".to_owned());
    opt_test!(FileOptions, php_metadata_namespace, "some value".to_owned());
    opt_test!(FileOptions, swift_prefix, "some value".to_owned());
    opt_test!(FileOptions, java_generic_services, true);
    opt_test!(FileOptions, java_outer_classname, "some value".to_owned());
    opt_test!(FileOptions, java_multiple_files, true);
    opt_test!(FileOptions, cc_generic_services, true);
    opt_test!(FileOptions, cc_enable_arenas, true);
    opt_test!(FileOptions, java_string_check_utf8, true);
    opt_test!(FileOptions, optimize_for, 123);
    opt_test!(FileOptions, php_generic_services, true);
    opt_test!(FileOptions, php_class_prefix, "some value".to_owned());
    opt_test!(FileOptions, py_generic_services, true);
    opt_test!(FileOptions, objc_class_prefix, "some value".to_owned());

    #[test]
    fn kv_option() -> Result<()> {
        let mut options = FileOptions::default();
        options.set_extension_data(
            proto_options::FILE_KEY_VALUE,
            vec!["test_key=some_value".to_owned()],
        )?;
        let context = file_with_options(options)?;
        test_file_script(
            context,
            "output.append(context.options[\"test_key\"]);",
            "some_value",
        )
    }

    fn run_test(options: FileOptions, method: &str, expected_output: &str) -> Result<()> {
        let context = file_with_options(options)?;
        test_file_script(
            context,
            &format!("output.append(context.options.{}.to_string());", method),
            expected_output,
        )
    }
}

mod enum_options {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::EnumOptions;

    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::integration_tests::{
        default_enum_proto, default_message_proto, file_with_enums, file_with_messages,
        test_file_script,
    };

    opt_test!(EnumOptions, deprecated, true);
    opt_test!(EnumOptions, allow_alias, true);

    #[test]
    fn kv_option() -> Result<()> {
        let mut options = EnumOptions::default();
        options.set_extension_data(
            proto_options::ENUM_KEY_VALUE,
            vec!["test_key=some_value".to_owned()],
        )?;
        let context = file_context(options)?;
        test_file_script(
            context,
            "output.append(context.enums[0].options[\"test_key\"]);",
            "some_value",
        )
    }

    fn run_test(options: EnumOptions, method: &str, expected_output: &str) -> Result<()> {
        let context = file_context(options)?;
        test_file_script(
            context,
            &format!(
                "output.append(context.enums[0].options.{}.to_string());",
                method
            ),
            expected_output,
        )
    }

    fn file_context(options: EnumOptions) -> Result<FileContext> {
        let mut proto = default_enum_proto("SomeEnum");
        proto.options = Some(options);
        file_with_enums(vec![proto])
    }
}

mod enum_value_options {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::{EnumOptions, EnumValueDescriptorProto, EnumValueOptions};

    use crate::renderer::context::{EnumValueContext, FileContext};
    use crate::renderer::scripted::integration_tests::{
        default_enum_proto, default_message_proto, file_with_enums, file_with_messages,
        test_file_script,
    };

    opt_test!(EnumValueOptions, deprecated, true);

    fn run_test(options: EnumValueOptions, method: &str, expected_output: &str) -> Result<()> {
        let context = file_context(options)?;
        test_file_script(
            context,
            &format!(
                "output.append(context.enums[0].values[0].options.{}.to_string());",
                method
            ),
            expected_output,
        )
    }

    fn file_context(options: EnumValueOptions) -> Result<FileContext> {
        let mut proto = default_enum_proto("SomeEnum");
        proto.value.push(EnumValueDescriptorProto {
            name: Some("SomeEnumValue".to_string()),
            number: Some(1),
            options: Some(options),
        });
        file_with_enums(vec![proto])
    }
}

mod message_options {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::MessageOptions;

    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::integration_tests::{
        default_message_proto, file_with_messages, test_file_script,
    };

    opt_test!(MessageOptions, message_set_wire_format, true);
    opt_test!(MessageOptions, no_standard_descriptor_accessor, true);
    opt_test!(MessageOptions, deprecated, true);
    opt_test!(MessageOptions, map_entry, true);

    #[test]
    fn kv_option() -> Result<()> {
        let mut options = MessageOptions::default();
        options.set_extension_data(
            proto_options::MSG_KEY_VALUE,
            vec!["test_key=some_value".to_owned()],
        )?;
        let context = file_context(options)?;
        test_file_script(
            context,
            "output.append(context.messages[0].options[\"test_key\"]);",
            "some_value",
        )
    }

    fn run_test(options: MessageOptions, method: &str, expected_output: &str) -> Result<()> {
        let context = file_context(options)?;
        test_file_script(
            context,
            &format!(
                "output.append(context.messages[0].options.{}.to_string());",
                method
            ),
            expected_output,
        )
    }

    fn file_context(options: MessageOptions) -> Result<FileContext> {
        let mut message = default_message_proto("SomeMessage");
        message.options = Some(options);
        file_with_messages(vec![message])
    }
}

mod field_options {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::{FieldOptions, FileDescriptorProto};

    use crate::renderer::context::FileContext;
    use crate::renderer::scripted::integration_tests::{
        default_field_proto, default_message_proto, file_with_messages, test_file_script,
    };

    opt_test!(FieldOptions, ctype, 1);
    opt_test!(FieldOptions, jstype, 2);
    opt_test!(FieldOptions, packed, true);
    opt_test!(FieldOptions, lazy, true);
    opt_test!(FieldOptions, deprecated, true);
    opt_test!(FieldOptions, weak, true);

    fn run_test(options: FieldOptions, method: &str, expected_output: &str) -> Result<()> {
        let context = file_context(options)?;
        test_file_script(
            context,
            &format!(
                "output.append(context.messages[0].fields[0].options.{}.to_string());",
                method
            ),
            expected_output,
        )
    }

    #[test]
    fn kv_option() -> Result<()> {
        let mut options = FieldOptions::default();
        options.set_extension_data(
            proto_options::FIELD_KEY_VALUE,
            vec!["test_key=some_value".to_owned()],
        )?;
        let context = file_context(options)?;
        test_file_script(
            context,
            "output.append(context.messages[0].fields[0].options[\"test_key\"]);",
            "some_value",
        )
    }

    fn file_context(options: FieldOptions) -> Result<FileContext> {
        let mut field = default_field_proto("some_field", "SomeType");
        field.options = Some(options);
        let mut message = default_message_proto("SomeMessage");
        message.field.push(field);
        file_with_messages(vec![message])
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

fn default_field_proto(name: &str, type_name: &str) -> FieldDescriptorProto {
    FieldDescriptorProto {
        name: Some(name.to_owned()),
        type_name: Some(type_name.to_owned()),
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

fn test_file_script(
    context: FileContext,
    script_content: &str,
    expected_output: &str,
) -> Result<()> {
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
    renderer.render_file(context, &mut buffer)?;
    assert_eq!(String::from_utf8(buffer).unwrap(), expected_output);
    Ok(())
}

fn test_metadata_script(
    context: MetadataContext,
    script_content: &str,
    expected_output: &str,
) -> Result<()> {
    let mut renderer = ScriptedRenderer::new();
    renderer.load_test_script(&format!(
        r#"
            fn render_metadata(context, output) {{
                {}
                output
            }}"#,
        script_content
    ))?;
    let mut buffer = Vec::new();
    renderer.render_metadata(context, &mut buffer)?;
    assert_eq!(String::from_utf8(buffer).unwrap(), expected_output);
    Ok(())
}
