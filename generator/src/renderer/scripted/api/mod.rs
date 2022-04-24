use crate::renderer::option_key_value::get_key_values;
use prost::{Extendable, ExtensionImpl};
use rhai;
use rhai::exported_module;
use rhai::plugin::*;
use std::collections::{BTreeMap, HashMap};

pub mod output;

pub fn register(engine: &mut rhai::Engine) {
    output::register(engine);
    register_context(engine);
    proto_options::register_script_apis(engine);
}

fn register_context(engine: &mut rhai::Engine) {
    engine.register_global_module(exported_module!(context).into());
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
mod context {
    use crate::renderer::context;
    use crate::util::DisplayNormalized;

    use super::get_str_or_new;

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
}
