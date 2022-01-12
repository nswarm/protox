use std::collections::HashMap;

use anyhow::Result;
use log::debug;
use prost_types::{FileDescriptorProto, FileOptions};
use serde::ser::Error;
use serde::{Deserialize, Serialize, Serializer};

use crate::template_renderer::context::{EnumContext, ImportContext, MessageContext};
use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;

#[derive(Serialize, Deserialize)]
pub struct FileContext<'a> {
    /// Relative path to the proto file this context is based on.
    source_file: &'a str,

    /// Other proto file imports of this proto file.
    imports: Vec<ImportContext>,

    /// Enums defined in this proto file.
    enums: Vec<EnumContext>,

    /// Messages defined in this proto file.
    messages: Vec<MessageContext>,

    /// Proto file options are serialized as an object like so:
    /// ```json
    /// {
    ///   "option_name": <option_value>,
    ///   ...etc.
    /// }
    /// ```
    /// Which can be accessed in the template like `{{options.option_name}}`. Options which have no
    /// value will not exist in the context, so you probably want to if guard:
    /// ```handlebars
    /// {{#if options.option_name}}
    ///   {{options.option_name}}
    /// {{/if}}
    /// ```
    /// Note that for boolean values one #if is enough to check both that it exists and is true.
    ///
    /// Built-in proto option names and types can be seen here:
    /// https://docs.rs/prost-types/latest/prost_types/struct.FileOptions.html
    ///
    /// Additionally, a few idlx-specific options are supported:
    ///
    #[serde(serialize_with = "serialize_file_options", skip_deserializing)]
    options: Option<FileOptions>,
}

impl<'a> FileContext<'a> {
    pub fn new(file: &'a FileDescriptorProto, config: &'a RendererConfig) -> Result<Self> {
        debug!(
            "Creating file context: {}",
            util::str_or_unknown(&file.name)
        );
        let context = Self {
            source_file: source_file(file)?,
            imports: imports(file)?,
            enums: enums(file, config)?,
            messages: messages(file, file.package.as_ref(), config)?,
            options: file.options.clone(),
        };
        Ok(context)
    }
}

fn source_file(file: &FileDescriptorProto) -> Result<&str> {
    util::str_or_error(&file.name, || "File has no 'name'".to_string())
}

fn imports(file: &FileDescriptorProto) -> Result<Vec<ImportContext>> {
    let mut imports = Vec::new();
    for import in &file.dependency {
        imports.push(ImportContext::new(import)?);
    }
    Ok(imports)
}

fn enums<'a>(
    file: &'a FileDescriptorProto,
    config: &'a RendererConfig,
) -> Result<Vec<EnumContext>> {
    let mut enums = Vec::new();
    for proto in &file.enum_type {
        enums.push(EnumContext::new(proto, config)?);
    }
    Ok(enums)
}

fn messages<'a>(
    file: &'a FileDescriptorProto,
    package: Option<&String>,
    config: &'a RendererConfig,
) -> Result<Vec<MessageContext>> {
    let mut messages = Vec::new();
    for message in &file.message_type {
        messages.push(MessageContext::new(message, package, config)?);
    }
    Ok(messages)
}

macro_rules! insert_file_option {
    ($name: ident, $map: ident, $opt: ident) => {
        try_insert_option($map, stringify!($name), &$opt.$name)?;
    };
}

fn serialize_file_options<S: Serializer>(
    options: &Option<FileOptions>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let options = match options {
        None => return serializer.serialize_none(),
        Some(options) => options,
    };
    let mut map = HashMap::new();
    if let Err(err) = insert_file_options(&mut map, options) {
        return Err(S::Error::custom(format!(
            "Error in serialize_file_options: {}",
            err
        )));
    }
    serializer.collect_map(map)
}

fn insert_file_options(
    map: &mut HashMap<String, serde_json::Value>,
    options: &FileOptions,
) -> Result<(), serde_json::Error> {
    // Built-in.
    insert_file_option!(deprecated, map, options);
    insert_file_option!(go_package, map, options);
    insert_file_option!(java_package, map, options);
    insert_file_option!(ruby_package, map, options);
    insert_file_option!(csharp_namespace, map, options);
    insert_file_option!(php_namespace, map, options);
    insert_file_option!(php_metadata_namespace, map, options);
    insert_file_option!(swift_prefix, map, options);
    insert_file_option!(java_generic_services, map, options);
    insert_file_option!(java_outer_classname, map, options);
    insert_file_option!(java_multiple_files, map, options);
    insert_file_option!(cc_generic_services, map, options);
    insert_file_option!(cc_enable_arenas, map, options);
    insert_file_option!(java_string_check_utf8, map, options);
    insert_file_option!(optimize_for, map, options);
    insert_file_option!(php_generic_services, map, options);
    insert_file_option!(php_class_prefix, map, options);
    insert_file_option!(py_generic_services, map, options);
    insert_file_option!(objc_class_prefix, map, options);
    Ok(())
}

fn try_insert_option<T: Serialize>(
    map: &mut HashMap<String, serde_json::Value>,
    name: impl Into<String>,
    value: &Option<T>,
) -> Result<(), serde_json::Error> {
    if let Some(value) = value.as_ref().map(|t| serde_json::to_value(&t)) {
        map.insert(name.into(), value?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use prost_types::{FileDescriptorProto, FileOptions};

    use crate::template_renderer::context::FileContext;
    use crate::template_renderer::renderer_config::RendererConfig;

    #[test]
    fn source_file() -> Result<()> {
        let config = RendererConfig::default();
        let name = "file_name".to_string();
        let mut file = default_file();
        file.name = Some(name.clone());
        let context = FileContext::new(&file, &config)?;
        assert_eq!(context.source_file, name);
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let file = default_file();
        let result = FileContext::new(&file, &config);
        assert!(result.is_err());
    }

    #[test]
    fn file_options() -> Result<()> {
        let config = RendererConfig::default();
        let mut file = default_file();
        file.name = Some("file_name".to_string());
        file.options = Some(
            #[allow(deprecated)]
            FileOptions {
                java_package: Some("java_package".to_string()),
                java_outer_classname: Some("java_outer_classname".to_string()),
                java_multiple_files: Some(true),
                java_generate_equals_and_hash: None,
                java_string_check_utf8: Some(true),
                optimize_for: Some(1234),
                go_package: Some("go_package".to_string()),
                cc_generic_services: Some(true),
                java_generic_services: Some(true),
                py_generic_services: Some(true),
                php_generic_services: Some(true),
                deprecated: Some(true),
                cc_enable_arenas: Some(true),
                objc_class_prefix: Some("objc_class_prefix".to_string()),
                csharp_namespace: Some("csharp_namespace".to_string()),
                swift_prefix: Some("swift_prefix".to_string()),
                php_class_prefix: Some("php_class_prefix".to_string()),
                php_namespace: Some("php_namespace".to_string()),
                php_metadata_namespace: Some("php_metadata_namespace".to_string()),
                ruby_package: Some("ruby_package".to_string()),
                uninterpreted_option: vec![],
            },
        );
        let context = FileContext::new(&file, &config)?;
        let json = serde_json::to_string(&context)?;
        println!("{}", json);
        assert!(json.contains(r#""java_package":"java_package""#));
        assert!(json.contains(r#""java_outer_classname":"java_outer_classname""#));
        assert!(json.contains(r#""java_multiple_files":true"#));
        assert!(json.contains(r#""java_string_check_utf8":true"#));
        assert!(json.contains(r#""optimize_for":1234"#));
        assert!(json.contains(r#""go_package":"go_package""#));
        assert!(json.contains(r#""cc_generic_services":true"#));
        assert!(json.contains(r#""java_generic_services":true"#));
        assert!(json.contains(r#""py_generic_services":true"#));
        assert!(json.contains(r#""php_generic_services":true"#));
        assert!(json.contains(r#""deprecated":true"#));
        assert!(json.contains(r#""cc_enable_arenas":true"#));
        assert!(json.contains(r#""objc_class_prefix":"objc_class_prefix""#));
        assert!(json.contains(r#""csharp_namespace":"csharp_namespace""#));
        assert!(json.contains(r#""swift_prefix":"swift_prefix""#));
        assert!(json.contains(r#""php_class_prefix":"php_class_prefix""#));
        assert!(json.contains(r#""php_namespace":"php_namespace""#));
        assert!(json.contains(r#""php_metadata_namespace":"php_metadata_namespace""#));
        assert!(json.contains(r#""ruby_package":"ruby_package""#));
        Ok(())
    }

    fn default_file() -> FileDescriptorProto {
        FileDescriptorProto {
            name: None,
            package: None,
            dependency: vec![],
            public_dependency: vec![],
            weak_dependency: vec![],
            message_type: vec![],
            enum_type: vec![],
            service: vec![],
            extension: vec![],
            options: None,
            source_code_info: None,
            syntax: None,
        }
    }
}
