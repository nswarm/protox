use crate::renderer::option_key_value::get_key_values;
use prost::{Extendable, ExtensionImpl};
use rhai::exported_module;
use rhai::plugin::*;
use std::collections::{BTreeMap, HashMap};

pub mod output;

pub fn register(engine: &mut Engine) {
    output::register(engine);
    register_context(engine);
    proto_options::register_script_apis(engine);
}

fn register_context(engine: &mut Engine) {
    engine.register_global_module(exported_module!(api).into());
}

fn get_str_or_new(opt: Option<&String>) -> String {
    opt.map(&String::clone).unwrap_or(String::new())
}

fn opt_get_kv<T: Extendable>(
    options: &T,
    index: String,
    ext: &ExtensionImpl<Vec<String>>,
) -> String {
    let kv = match get_key_values(options, &ext) {
        Err(err) => panic!("Error getting kv option '{}': {}", index, err),
        Ok(kv) => kv,
    };
    for (key, value) in kv {
        if key == index {
            return value.to_owned();
        }
    }
    String::new()
}

fn hash_to_btree<K: Ord, V>(map: HashMap<K, V>) -> BTreeMap<K, V> {
    let mut btree = BTreeMap::<K, V>::new();
    for (k, v) in map {
        btree.insert(k, v);
    }
    btree
}

#[export_module]
mod api {
    use super::get_str_or_new;
    use crate::renderer::context;
    use crate::renderer::context::overlayed::Overlayed;
    use crate::renderer::scripted::api::{hash_to_btree, opt_get_kv};
    use crate::util::DisplayNormalized;
    use log::error;
    use std::collections::BTreeMap;

    ////////////////////////////////////////////////////
    // Utilities

    #[rhai_fn(name = "join", pure)]
    pub fn array_join(array: &mut rhai::Array, separator: &str) -> String {
        let mut result = String::new();
        for (i, item) in array.iter().enumerate() {
            result.push_str(&item.to_string());
            if i < array.len() - 1 {
                result.push_str(separator);
            }
        }
        return result;
    }

    pub type YamlValue = serde_yaml::Value;

    ////////////////////////////////////////////////////
    // Contexts

    pub type FileContext = context::FileContext;
    pub type ImportContext = context::ImportContext;
    pub type EnumContext = context::EnumContext;
    pub type EnumValueContext = context::EnumValueContext;
    pub type MessageContext = context::MessageContext;
    pub type FieldContext = context::FieldContext;

    pub type MetadataContext = context::MetadataContext;
    pub type PackageFile = context::PackageFile;
    pub type PackageTreeNode = context::PackageTreeNode;

    pub type FileOptions = prost_types::FileOptions;
    pub type EnumOptions = prost_types::EnumOptions;
    pub type EnumValueOptions = prost_types::EnumValueOptions;
    pub type MessageOptions = prost_types::MessageOptions;
    pub type FieldOptions = prost_types::FieldOptions;

    ////////////////////////////////////////////////////
    // FileContext
    #[rhai_fn(get = "source_file", pure)]
    pub fn file_source_file(context: &mut FileContext) -> String {
        context.source_file().to_owned()
    }
    #[rhai_fn(get = "package_", pure)]
    pub fn file_package(context: &mut FileContext) -> String {
        context.package().to_owned()
    }
    #[rhai_fn(get = "imports", pure)]
    pub fn file_imports(context: &mut FileContext) -> rhai::Dynamic {
        context.imports().clone().into()
    }
    #[rhai_fn(get = "enums", pure)]
    pub fn file_enums(context: &mut FileContext) -> rhai::Dynamic {
        context.enums().clone().into()
    }
    #[rhai_fn(get = "messages", pure)]
    pub fn file_messages(context: &mut FileContext) -> rhai::Dynamic {
        context.messages().clone().into()
    }
    #[rhai_fn(get = "options", pure)]
    pub fn file_options(context: &mut FileContext) -> FileOptions {
        context.options().clone().unwrap_or(FileOptions::default())
    }

    #[rhai_fn(name = "overlay")]
    pub fn file_overlay(context: &mut FileContext, key: String) -> serde_yaml::Value {
        context.overlay(&key)
    }

    ////////////////////////////////////////////////////
    // ImportContext
    #[rhai_fn(get = "file_path", pure)]
    pub fn import_file_path(context: &mut ImportContext) -> String {
        context.file_path().to_owned()
    }

    #[rhai_fn(get = "file_name", pure)]
    pub fn import_file_name(context: &mut ImportContext) -> String {
        context.file_name().to_owned()
    }

    #[rhai_fn(get = "file_name_with_ext", pure)]
    pub fn import_file_name_with_ext(context: &mut ImportContext) -> String {
        context.file_name_with_ext().to_owned()
    }

    ////////////////////////////////////////////////////
    // EnumContext
    #[rhai_fn(get = "name", pure)]
    pub fn enum_name(context: &mut EnumContext) -> String {
        context.name().to_owned()
    }

    #[rhai_fn(get = "values", pure)]
    pub fn enum_values(context: &mut EnumContext) -> rhai::Dynamic {
        context.values().clone().into()
    }

    #[rhai_fn(get = "options", pure)]
    pub fn enum_options(context: &mut EnumContext) -> EnumOptions {
        context.options().clone().unwrap_or(EnumOptions::default())
    }

    #[rhai_fn(name = "overlay")]
    pub fn enum_overlay(context: &mut EnumContext, key: String) -> YamlValue {
        context.overlay(&key)
    }

    ////////////////////////////////////////////////////
    // EnumValueContext
    #[rhai_fn(get = "name", pure)]
    pub fn enum_value_name(context: &mut EnumValueContext) -> String {
        context.name().to_owned()
    }

    #[rhai_fn(get = "number", pure)]
    pub fn enum_value_number(context: &mut EnumValueContext) -> rhai::INT {
        context.number().into()
    }

    #[rhai_fn(get = "options", pure)]
    pub fn enum_value_options(context: &mut EnumValueContext) -> EnumValueOptions {
        context
            .options()
            .clone()
            .unwrap_or(EnumValueOptions::default())
    }

    #[rhai_fn(name = "overlay")]
    pub fn enum_value_overlay(context: &mut EnumValueContext, key: String) -> YamlValue {
        context.overlay(&key)
    }

    ////////////////////////////////////////////////////
    // MessageContext
    #[rhai_fn(get = "name", pure)]
    pub fn message_name(context: &mut MessageContext) -> String {
        context.name().to_owned()
    }

    #[rhai_fn(get = "fields", pure)]
    pub fn message_fields(context: &mut MessageContext) -> rhai::Dynamic {
        context.fields().clone().into()
    }

    #[rhai_fn(get = "options", pure)]
    pub fn message_options(context: &mut MessageContext) -> MessageOptions {
        context
            .options()
            .clone()
            .unwrap_or(MessageOptions::default())
    }

    #[rhai_fn(name = "overlay")]
    pub fn message_overlay(context: &mut MessageContext, key: String) -> YamlValue {
        context.overlay(&key)
    }

    ////////////////////////////////////////////////////
    // FieldContext
    #[rhai_fn(get = "name", pure)]
    pub fn field_name(context: &mut FieldContext) -> String {
        context.name().to_owned()
    }

    #[rhai_fn(get = "fully_qualified_type", pure)]
    pub fn field_fully_qualified_type(context: &mut FieldContext) -> String {
        get_str_or_new(context.fully_qualified_type())
    }

    #[rhai_fn(get = "relative_type", pure)]
    pub fn field_relative_type(context: &mut FieldContext) -> String {
        get_str_or_new(context.relative_type())
    }

    #[rhai_fn(get = "is_array", pure)]
    pub fn field_is_array(context: &mut FieldContext) -> bool {
        context.is_array()
    }

    #[rhai_fn(get = "is_map", pure)]
    pub fn field_is_map(context: &mut FieldContext) -> bool {
        context.is_map()
    }

    #[rhai_fn(get = "is_oneof", pure)]
    pub fn field_is_oneof(context: &mut FieldContext) -> bool {
        context.is_oneof()
    }

    #[rhai_fn(get = "fully_qualified_key_type", pure)]
    pub fn field_fully_qualified_key_type(context: &mut FieldContext) -> String {
        get_str_or_new(context.fully_qualified_key_type())
    }

    #[rhai_fn(get = "fully_qualified_value_type", pure)]
    pub fn field_fully_qualified_value_type(context: &mut FieldContext) -> String {
        get_str_or_new(context.fully_qualified_value_type())
    }

    #[rhai_fn(get = "relative_key_type", pure)]
    pub fn field_relative_key_type(context: &mut FieldContext) -> String {
        get_str_or_new(context.relative_key_type())
    }

    #[rhai_fn(get = "relative_value_type", pure)]
    pub fn field_relative_value_type(context: &mut FieldContext) -> String {
        get_str_or_new(context.relative_value_type())
    }

    #[rhai_fn(get = "options", pure)]
    pub fn field_options(context: &mut FieldContext) -> FieldOptions {
        context
            .options()
            .map(FieldOptions::clone)
            .unwrap_or(FieldOptions::default())
    }

    #[rhai_fn(name = "overlay")]
    pub fn field_overlay(context: &mut FieldContext, key: String) -> YamlValue {
        context.overlay(&key)
    }

    ////////////////////////////////////////////////////
    // MetadataContext

    #[rhai_fn(get = "directory", pure)]
    pub fn metadata_directory(context: &mut MetadataContext) -> String {
        context.directory().display_normalized()
    }

    #[rhai_fn(get = "file_names", pure)]
    pub fn metadata_file_names(context: &mut MetadataContext) -> rhai::Dynamic {
        context.file_names().clone().into()
    }

    #[rhai_fn(get = "file_names_with_ext", pure)]
    pub fn metadata_file_names_with_ext(context: &mut MetadataContext) -> rhai::Dynamic {
        context.file_names_with_ext().to_vec().into()
    }

    #[rhai_fn(get = "subdirectories", pure)]
    pub fn metadata_subdirectories(context: &mut MetadataContext) -> rhai::Dynamic {
        context.subdirectories().to_vec().into()
    }

    #[rhai_fn(get = "package_files_full", pure)]
    pub fn metadata_package_files_full(context: &mut MetadataContext) -> rhai::Dynamic {
        context.package_files_full().to_vec().into()
    }

    #[rhai_fn(get = "package_file_tree", pure)]
    pub fn metadata_package_file_tree(context: &mut MetadataContext) -> rhai::Dynamic {
        hash_to_btree(context.package_file_tree().clone()).into()
    }

    ////////////////////////////////////////////////////
    // PackageFile

    #[rhai_fn(get = "file_name", pure)]
    pub fn package_file_file_name(context: &mut PackageFile) -> String {
        context.file_name().to_owned()
    }

    #[rhai_fn(get = "file_package", pure)]
    pub fn package_file_package(context: &mut PackageFile) -> String {
        context.package().to_owned()
    }

    ////////////////////////////////////////////////////
    // PackageTreeNode

    #[rhai_fn(get = "file_name", pure)]
    pub fn package_tree_node_file_name(context: &mut PackageTreeNode) -> String {
        get_str_or_new(context.file_name())
    }

    #[rhai_fn(get = "children", pure)]
    pub fn package_tree_node_children(context: &mut PackageTreeNode) -> rhai::Dynamic {
        hash_to_btree(context.children().clone()).into()
    }

    ////////////////////////////////////////////////////
    // FileOptions

    // Built-in.
    #[rhai_fn(get = "deprecated", pure)]
    pub fn file_opt_deprecated(opt: &mut FileOptions) -> bool {
        opt.deprecated.unwrap_or(false)
    }
    #[rhai_fn(get = "go_package", pure)]
    pub fn file_opt_go_package(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.go_package.as_ref())
    }
    #[rhai_fn(get = "java_package", pure)]
    pub fn file_opt_java_package(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.java_package.as_ref())
    }
    #[rhai_fn(get = "ruby_package", pure)]
    pub fn file_opt_ruby_package(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.ruby_package.as_ref())
    }
    #[rhai_fn(get = "csharp_namespace", pure)]
    pub fn file_opt_csharp_namespace(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.csharp_namespace.as_ref())
    }
    #[rhai_fn(get = "php_namespace", pure)]
    pub fn file_opt_php_namespace(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.php_namespace.as_ref())
    }
    #[rhai_fn(get = "php_metadata_namespace", pure)]
    pub fn file_opt_php_metadata_namespace(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.php_metadata_namespace.as_ref())
    }
    #[rhai_fn(get = "swift_prefix", pure)]
    pub fn file_opt_swift_prefix(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.swift_prefix.as_ref())
    }
    #[rhai_fn(get = "java_generic_services", pure)]
    pub fn file_opt_java_generic_services(opt: &mut FileOptions) -> bool {
        opt.java_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "java_outer_classname", pure)]
    pub fn file_opt_java_outer_classname(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.java_outer_classname.as_ref())
    }
    #[rhai_fn(get = "java_multiple_files", pure)]
    pub fn file_opt_java_multiple_files(opt: &mut FileOptions) -> bool {
        opt.java_multiple_files.unwrap_or(false)
    }
    #[rhai_fn(get = "cc_generic_services", pure)]
    pub fn file_opt_cc_generic_services(opt: &mut FileOptions) -> bool {
        opt.cc_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "cc_enable_arenas", pure)]
    pub fn file_opt_cc_enable_arenas(opt: &mut FileOptions) -> bool {
        opt.cc_enable_arenas.unwrap_or(false)
    }
    #[rhai_fn(get = "java_string_check_utf8", pure)]
    pub fn file_opt_java_string_check_utf8(opt: &mut FileOptions) -> bool {
        opt.java_string_check_utf8.unwrap_or(false)
    }
    #[rhai_fn(get = "optimize_for", pure)]
    pub fn file_opt_optimize_for(opt: &mut FileOptions) -> rhai::INT {
        // Default is from protobuf.
        opt.optimize_for
            .unwrap_or(prost_types::file_options::OptimizeMode::Speed as i32) as rhai::INT
    }
    #[rhai_fn(get = "php_generic_services", pure)]
    pub fn file_opt_php_generic_services(opt: &mut FileOptions) -> bool {
        opt.php_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "php_class_prefix", pure)]
    pub fn file_opt_php_class_prefix(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.php_class_prefix.as_ref())
    }
    #[rhai_fn(get = "py_generic_services", pure)]
    pub fn file_opt_py_generic_services(opt: &mut FileOptions) -> bool {
        opt.py_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "objc_class_prefix", pure)]
    pub fn file_opt_objc_class_prefix(opt: &mut FileOptions) -> String {
        get_str_or_new(opt.objc_class_prefix.as_ref())
    }

    // Key-value custom proto options.
    #[rhai_fn(index_get)]
    pub fn file_opt_get_kv(options: &mut FileOptions, index: String) -> String {
        opt_get_kv(options, index, &proto_options::FILE_KEY_VALUE)
    }

    ////////////////////////////////////////////////////
    // EnumOptions
    #[rhai_fn(get = "allow_alias", pure)]
    pub fn enum_opt_allow_alias(opt: &mut EnumOptions) -> bool {
        opt.allow_alias.unwrap_or(false)
    }
    #[rhai_fn(get = "deprecated", pure)]
    pub fn enum_opt_deprecated(opt: &mut EnumOptions) -> bool {
        opt.deprecated.unwrap_or(false)
    }

    // Key-value custom proto options.
    #[rhai_fn(index_get)]
    pub fn enum_opt_get_kv(options: &mut EnumOptions, index: String) -> String {
        opt_get_kv(options, index, &proto_options::ENUM_KEY_VALUE)
    }

    ////////////////////////////////////////////////////
    // EnumValueOptions
    #[rhai_fn(get = "deprecated", pure)]
    pub fn enum_value_opt_deprecated(opt: &mut EnumValueOptions) -> bool {
        opt.deprecated.unwrap_or(false)
    }

    ////////////////////////////////////////////////////
    // MessageOptions
    #[rhai_fn(get = "message_set_wire_format", pure)]
    pub fn message_opt_message_set_wire_format(opt: &mut MessageOptions) -> bool {
        opt.message_set_wire_format.unwrap_or(false)
    }
    #[rhai_fn(get = "no_standard_descriptor_accessor", pure)]
    pub fn message_opt_no_standard_descriptor_accessor(opt: &mut MessageOptions) -> bool {
        opt.no_standard_descriptor_accessor.unwrap_or(false)
    }
    #[rhai_fn(get = "deprecated", pure)]
    pub fn message_opt_deprecated(opt: &mut MessageOptions) -> bool {
        opt.deprecated.unwrap_or(false)
    }
    #[rhai_fn(get = "map_entry", pure)]
    pub fn message_opt_map_entry(opt: &mut MessageOptions) -> bool {
        opt.map_entry.unwrap_or(false)
    }

    // Key-value custom proto options.
    #[rhai_fn(index_get)]
    pub fn message_opt_get_kv(options: &mut MessageOptions, index: String) -> String {
        opt_get_kv(options, index, &proto_options::MSG_KEY_VALUE)
    }

    ////////////////////////////////////////////////////
    // FieldOptions
    #[rhai_fn(get = "ctype", pure)]
    pub fn field_opt_ctype(opt: &mut FieldOptions) -> rhai::INT {
        opt.ctype.unwrap_or(0) as rhai::INT
    }
    #[rhai_fn(get = "jstype", pure)]
    pub fn field_opt_jstype(opt: &mut FieldOptions) -> rhai::INT {
        opt.jstype.unwrap_or(0) as rhai::INT
    }
    #[rhai_fn(get = "packed", pure)]
    pub fn field_opt_packed(opt: &mut FieldOptions) -> bool {
        opt.packed.unwrap_or(false)
    }
    #[rhai_fn(get = "lazy", pure)]
    pub fn field_opt_lazy(opt: &mut FieldOptions) -> bool {
        opt.lazy.unwrap_or(false)
    }
    #[rhai_fn(get = "deprecated", pure)]
    pub fn field_opt_deprecated(opt: &mut FieldOptions) -> bool {
        opt.deprecated.unwrap_or(false)
    }
    #[rhai_fn(get = "weak", pure)]
    pub fn field_opt_weak(opt: &mut FieldOptions) -> bool {
        opt.weak.unwrap_or(false)
    }

    // Key-value custom proto options.
    #[rhai_fn(index_get)]
    pub fn field_opt_get_kv(options: &mut FieldOptions, index: String) -> String {
        opt_get_kv(options, index, &proto_options::FIELD_KEY_VALUE)
    }

    ////////////////////////////////////////////////////
    // Value
    #[rhai_fn(name = "is_null", pure)]
    pub fn yaml_value_is_null(value: &mut YamlValue) -> bool {
        value.is_null()
    }

    #[rhai_fn(name = "is_valid", pure)]
    pub fn yaml_value_is_valid(value: &mut YamlValue) -> bool {
        !value.is_null()
    }

    #[rhai_fn(name = "is_str", pure)]
    pub fn yaml_value_is_str(value: &mut YamlValue) -> bool {
        value.is_string()
    }

    #[rhai_fn(name = "is_string", pure)]
    pub fn yaml_value_is_string(value: &mut YamlValue) -> bool {
        value.is_string()
    }

    #[rhai_fn(name = "is_int", pure)]
    pub fn yaml_value_is_int(value: &mut YamlValue) -> bool {
        value.is_i64()
    }

    #[rhai_fn(name = "is_float", pure)]
    pub fn yaml_value_is_float(value: &mut YamlValue) -> bool {
        value.is_f64()
    }

    #[rhai_fn(name = "is_bool", pure)]
    pub fn yaml_value_is_bool(value: &mut YamlValue) -> bool {
        value.is_bool()
    }

    #[rhai_fn(name = "is_array", pure)]
    pub fn yaml_value_is_array(value: &mut YamlValue) -> bool {
        value.is_sequence()
    }

    #[rhai_fn(name = "is_map", pure)]
    pub fn yaml_value_is_map(value: &mut YamlValue) -> bool {
        value.is_mapping()
    }

    #[rhai_fn(name = "as_str", pure, return_raw)]
    pub fn yaml_value_as_str(value: &mut YamlValue) -> Result<String, Box<rhai::EvalAltResult>> {
        Ok(value
            .as_str()
            .map(|x| x.to_owned())
            .ok_or("value is not a string")?)
    }

    #[rhai_fn(name = "as_string", pure, return_raw)]
    pub fn yaml_value_as_string(value: &mut YamlValue) -> Result<String, Box<rhai::EvalAltResult>> {
        Ok(value
            .as_str()
            .map(|x| x.to_owned())
            .ok_or("value is not a string")?)
    }

    #[rhai_fn(name = "as_int", pure, return_raw)]
    pub fn yaml_value_as_int(value: &mut YamlValue) -> Result<rhai::INT, Box<rhai::EvalAltResult>> {
        Ok(value.as_i64().ok_or("value is not an i64")?)
    }

    #[rhai_fn(name = "as_float", pure, return_raw)]
    pub fn yaml_value_as_float(
        value: &mut YamlValue,
    ) -> Result<rhai::FLOAT, Box<rhai::EvalAltResult>> {
        Ok(value.as_f64().ok_or("value is not an f64")?)
    }

    #[rhai_fn(name = "as_bool", pure, return_raw)]
    pub fn yaml_value_as_bool(value: &mut YamlValue) -> Result<bool, Box<rhai::EvalAltResult>> {
        Ok(value.as_bool().ok_or("value is not a bool")?)
    }

    #[rhai_fn(name = "as_array", pure, return_raw)]
    pub fn yaml_value_as_array(
        value: &mut YamlValue,
    ) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        Ok(value
            .as_sequence()
            .ok_or("value is not an array")?
            .to_vec()
            .into())
    }

    #[rhai_fn(name = "as_map", pure, return_raw)]
    pub fn yaml_value_as_map(
        value: &mut YamlValue,
    ) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut map = BTreeMap::<String, YamlValue>::new();
        for (key, value) in value.as_mapping().ok_or("value is not an i64")? {
            if !key.is_string() {
                error!(
                    "Yaml maps with keys that are not Strings are unsupported. key: {:?}",
                    key
                );
                continue;
            }
            map.insert(key.as_str().unwrap().to_owned(), value.clone());
        }
        Ok(map.into())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rhai::exported_module;

    mod yaml {
        use crate::renderer::scripted::api::tests::run_test;
        use anyhow::Result;
        use std::collections::BTreeMap;

        #[test]
        fn is_valid() -> Result<()> {
            test_is_x(
                serde_yaml::Value::String("some_value".to_owned()),
                "value.is_valid()",
                true,
            )?;
            test_is_x(
                serde_yaml::Value::Number(5.into()),
                "value.is_valid()",
                true,
            )?;
            test_is_x(serde_yaml::Value::Null, "value.is_valid()", false)
        }

        #[test]
        fn is_null() -> Result<()> {
            test_is_x(
                serde_yaml::Value::String("some_value".to_owned()),
                "value.is_null()",
                false,
            )?;
            test_is_x(
                serde_yaml::Value::Number(5.into()),
                "value.is_null()",
                false,
            )?;
            test_is_x(serde_yaml::Value::Null, "value.is_null()", true)
        }

        #[test]
        fn is_str() -> Result<()> {
            test_is_x(
                serde_yaml::Value::String("some_value".to_owned()),
                "value.is_str()",
                true,
            )
        }

        #[test]
        fn is_not_str() -> Result<()> {
            test_is_x(serde_yaml::Value::Number(5.into()), "value.is_str()", false)
        }

        #[test]
        fn is_bool() -> Result<()> {
            test_is_x(serde_yaml::Value::Bool(true), "value.is_bool()", true)
        }

        #[test]
        fn is_not_bool() -> Result<()> {
            test_is_x(
                serde_yaml::Value::String("some_value".to_owned()),
                "value.is_bool()",
                false,
            )
        }

        #[test]
        fn is_int() -> Result<()> {
            test_is_x(serde_yaml::Value::Number(5.into()), "value.is_int()", true)
        }

        #[test]
        fn is_not_int() -> Result<()> {
            test_is_x(serde_yaml::Value::Bool(true), "value.is_int()", false)
        }

        #[test]
        fn is_array() -> Result<()> {
            test_is_x(
                serde_yaml::Value::Sequence(serde_yaml::Sequence::new()),
                "value.is_array()",
                true,
            )
        }

        #[test]
        fn is_not_array() -> Result<()> {
            test_is_x(
                serde_yaml::Value::Number(5.into()),
                "value.is_array()",
                false,
            )
        }

        #[test]
        fn is_map() -> Result<()> {
            test_is_x(
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                "value.is_map()",
                true,
            )
        }

        #[test]
        fn is_not_map() -> Result<()> {
            test_is_x(serde_yaml::Value::Number(5.into()), "value.is_map()", false)
        }

        fn test_is_x(value: serde_yaml::Value, script: &str, expected: bool) -> Result<()> {
            let actual = run_test::<bool>(value, script)?;
            assert_eq!(actual, expected);
            Ok(())
        }

        #[test]
        fn as_str() -> Result<()> {
            let result = run_test::<String>(
                serde_yaml::Value::String("hello".to_owned()),
                "value.as_str()",
            )?;
            assert_eq!(result, "hello".to_owned());
            Ok(())
        }

        #[test]
        fn as_bool() -> Result<()> {
            let result = run_test::<bool>(serde_yaml::Value::Bool(true), "value.as_bool()")?;
            assert!(result);
            Ok(())
        }

        #[test]
        fn as_int() -> Result<()> {
            let result =
                run_test::<rhai::INT>(serde_yaml::Value::Number(5.into()), "value.as_int()")?;
            assert_eq!(result, 5);
            Ok(())
        }

        #[test]
        fn as_float() -> Result<()> {
            let result =
                run_test::<rhai::FLOAT>(serde_yaml::Value::Number(5.5.into()), "value.as_float()")?;
            assert_eq!(result, 5.5);
            Ok(())
        }

        #[test]
        fn as_array() -> Result<()> {
            let expected = vec![
                serde_yaml::Value::Number(1.into()),
                serde_yaml::Value::Number(2.into()),
            ];
            let success = run_test::<bool>(
                serde_yaml::Value::Sequence(expected.clone()),
                r#"
                let arr = value.as_array();
                arr[0].as_int() == 1
                && arr[1].as_int() == 2
                "#,
            )?;
            assert!(success);
            Ok(())
        }

        #[test]
        fn as_map() -> Result<()> {
            let mut expected = BTreeMap::new();
            expected.insert("a".to_owned(), serde_yaml::Value::Number(1.into()));
            expected.insert("b".to_owned(), serde_yaml::Value::Number(2.into()));
            let success = run_test::<bool>(
                btree_to_mapping(expected.clone()),
                r#"
                let map = value.as_map();
                map.get("a").as_int() == 1
                && map.get("b").as_int() == 2
            "#,
            )?;
            assert!(success);
            Ok(())
        }

        fn btree_to_mapping(map: BTreeMap<String, serde_yaml::Value>) -> serde_yaml::Value {
            let mut mapping = serde_yaml::Mapping::new();
            for (k, v) in map {
                mapping.insert(serde_yaml::Value::String(k), v);
            }
            serde_yaml::Value::Mapping(mapping)
        }
    }

    fn run_test<T: 'static + Send + Sync + Clone>(
        value: serde_yaml::Value,
        script_content: &str,
    ) -> Result<T> {
        let mut engine = rhai::Engine::new();
        engine.register_global_module(exported_module!(super::api).into());
        let ast = engine.compile(&format!(
            r#"
        fn test(value) {{
            {}
        }}
        "#,
            script_content
        ))?;
        let mut scope = rhai::Scope::new();
        let ret_val: T = engine.call_fn(&mut scope, &ast, "test", (value,))?;
        Ok(ret_val)
    }
}
