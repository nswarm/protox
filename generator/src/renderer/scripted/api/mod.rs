use rhai;
use rhai::exported_module;
use rhai::plugin::*;

use crate::renderer::scripted::api::output::Output;

pub mod output;

pub fn register(engine: &mut rhai::Engine) {
    output::register(engine);
    context(engine);
}

fn context(engine: &mut rhai::Engine) {
    engine.register_global_module(exported_module!(context).into());
}

fn get_str_or_new(opt: &Option<String>) -> String {
    opt.as_ref().map(&String::clone).unwrap_or(String::new())
}

#[export_module]
mod context {
    use crate::renderer::context;

    use super::get_str_or_new;

    pub type FileContext = context::FileContext;
    pub type ImportContext = context::ImportContext;
    pub type EnumContext = context::EnumContext;
    pub type EnumValueContext = context::EnumValueContext;
    pub type MessageContext = context::MessageContext;
    pub type FieldContext = context::FieldContext;
    pub type MetadataContext = context::MetadataContext;

    pub type FileOptions = prost_types::FileOptions;

    ////////////////////////////////////////////////////
    // FileContext

    #[rhai_fn(get = "source_file")]
    pub fn source_file(context: &mut FileContext) -> String {
        context.source_file().to_owned()
    }
    #[rhai_fn(get = "imports")]
    pub fn imports(context: &mut FileContext) -> rhai::Dynamic {
        context.imports().clone().into()
    }
    #[rhai_fn(get = "enums")]
    pub fn enums(context: &mut FileContext) -> rhai::Dynamic {
        context.enums().clone().into()
    }
    #[rhai_fn(get = "messages")]
    pub fn messages(context: &mut FileContext) -> rhai::Dynamic {
        context.messages().clone().into()
    }
    #[rhai_fn(get = "options")]
    pub fn options(context: &mut FileContext) -> FileOptions {
        context.options().clone().unwrap_or(FileOptions::default())
    }

    ////////////////////////////////////////////////////
    // ImportContext

    #[rhai_fn(get = "file_path")]
    pub fn file_path(context: &mut ImportContext) -> String {
        context.file_path().to_owned()
    }

    #[rhai_fn(get = "file_name")]
    pub fn file_name(context: &mut ImportContext) -> String {
        context.file_name().to_owned()
    }

    #[rhai_fn(get = "file_name_with_ext")]
    pub fn file_name_with_ext(context: &mut ImportContext) -> String {
        context.file_name_with_ext().to_owned()
    }

    ////////////////////////////////////////////////////
    // EnumContext

    #[rhai_fn(get = "name")]
    pub fn enum_name(context: &mut EnumContext) -> String {
        context.name().to_owned()
    }

    ////////////////////////////////////////////////////
    // MessageContext

    #[rhai_fn(get = "name")]
    pub fn msg_name(context: &mut MessageContext) -> String {
        context.name().to_owned()
    }

    ////////////////////////////////////////////////////
    // FieldContext

    ////////////////////////////////////////////////////
    // MetadataContext

    ////////////////////////////////////////////////////
    // FileOptions

    // Built-in.

    #[rhai_fn(get = "deprecated")]
    pub fn file_opt_deprecated(opt: &mut FileOptions) -> bool {
        opt.deprecated.unwrap_or(false)
    }
    #[rhai_fn(get = "go_package")]
    pub fn file_opt_go_package(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.go_package)
    }
    #[rhai_fn(get = "java_package")]
    pub fn file_opt_java_package(opt: &mut FileOptions) -> String {
        println!(
            "---------- {}",
            opt.java_package.as_ref().unwrap_or(&"aaaaaaaaa".to_owned())
        );
        get_str_or_new(&opt.java_package)
    }
    #[rhai_fn(get = "ruby_package")]
    pub fn file_opt_ruby_package(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.ruby_package)
    }
    #[rhai_fn(get = "csharp_namespace")]
    pub fn file_opt_csharp_namespace(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.csharp_namespace)
    }
    #[rhai_fn(get = "php_namespace")]
    pub fn file_opt_php_namespace(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.php_namespace)
    }
    #[rhai_fn(get = "php_metadata_namespace")]
    pub fn file_opt_php_metadata_namespace(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.php_metadata_namespace)
    }
    #[rhai_fn(get = "swift_prefix")]
    pub fn file_opt_swift_prefix(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.swift_prefix)
    }
    #[rhai_fn(get = "java_generic_services")]
    pub fn file_opt_java_generic_services(opt: &mut FileOptions) -> bool {
        opt.java_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "java_outer_classname")]
    pub fn file_opt_java_outer_classname(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.java_outer_classname)
    }
    #[rhai_fn(get = "java_multiple_files")]
    pub fn file_opt_java_multiple_files(opt: &mut FileOptions) -> bool {
        opt.java_multiple_files.unwrap_or(false)
    }
    #[rhai_fn(get = "cc_generic_services")]
    pub fn file_opt_cc_generic_services(opt: &mut FileOptions) -> bool {
        opt.cc_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "cc_enable_arenas")]
    pub fn file_opt_cc_enable_arenas(opt: &mut FileOptions) -> bool {
        opt.cc_enable_arenas.unwrap_or(false)
    }
    #[rhai_fn(get = "java_string_check_utf8")]
    pub fn file_opt_java_string_check_utf8(opt: &mut FileOptions) -> bool {
        opt.java_string_check_utf8.unwrap_or(false)
    }
    #[rhai_fn(get = "optimize_for")]
    pub fn file_opt_optimize_for(opt: &mut FileOptions) -> i64 {
        // Default is from protobuf.
        opt.optimize_for
            .unwrap_or(prost_types::file_options::OptimizeMode::Speed as i32) as i64
    }
    #[rhai_fn(get = "php_generic_services")]
    pub fn file_opt_php_generic_services(opt: &mut FileOptions) -> bool {
        opt.php_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "php_class_prefix")]
    pub fn file_opt_php_class_prefix(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.php_class_prefix)
    }
    #[rhai_fn(get = "py_generic_services")]
    pub fn file_opt_py_generic_services(opt: &mut FileOptions) -> bool {
        opt.py_generic_services.unwrap_or(false)
    }
    #[rhai_fn(get = "objc_class_prefix")]
    pub fn file_opt_objc_class_prefix(opt: &mut FileOptions) -> String {
        get_str_or_new(&opt.objc_class_prefix)
    }

    // Key-value custom proto options.

    // #[rhai_fn(index_get)]
    // pub fn file_opt_get_kv(options: &mut FileOptions, index: String) -> String {
    //     String::new()
    //     // todo probably make these like... options.kv.abc? if we can
    //     // let json = context::serialize_file_options(&Some(options));
    // }
}
