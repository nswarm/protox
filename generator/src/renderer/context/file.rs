use std::collections::HashMap;

use anyhow::{anyhow, Result};
use log::debug;
use prost_types::{FileDescriptorProto, FileOptions};
use serde::ser::Error;
use serde::{Deserialize, Serialize, Serializer};

use crate::renderer::context::{EnumContext, ImportContext, MessageContext};
use crate::renderer::option_key_value::insert_custom_options;
use crate::renderer::proto::TypePath;
use crate::renderer::RendererConfig;
use crate::util;

#[derive(Serialize, Deserialize, Clone)]
pub struct FileContext {
    /// Relative path to the proto file this context is based on.
    source_file: String,

    /// Package defined in the file.
    package: String,

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
    /// Additionally, a few protox-specific options are supported. See the proto files at
    /// `protox/proto_options/protos` for more info.
    #[serde(serialize_with = "serialize_file_options", skip_deserializing)]
    options: Option<FileOptions>,
}

impl FileContext {
    pub fn new(file: &FileDescriptorProto, config: &RendererConfig) -> Result<Self> {
        debug!(
            "Creating file context: {}",
            util::str_or_unknown(&file.name)
        );
        let context = Self {
            source_file: source_file(file)?,
            package: package(file, &config),
            imports: imports(file, &config.ignored_imports)?,
            enums: enums(file, config)?,
            messages: messages(file, file.package.as_ref(), config)?,
            options: file.options.clone(),
        };
        Ok(context)
    }

    pub fn source_file(&self) -> &str {
        &self.source_file
    }
    pub fn imports(&self) -> &Vec<ImportContext> {
        &self.imports
    }
    pub fn enums(&self) -> &Vec<EnumContext> {
        &self.enums
    }
    pub fn messages(&self) -> &Vec<MessageContext> {
        &self.messages
    }
    pub fn options(&self) -> &Option<FileOptions> {
        &self.options
    }
}

fn source_file(file: &FileDescriptorProto) -> Result<String> {
    file.name
        .clone()
        .ok_or(anyhow!("File has no 'name'".to_owned()))
}

fn package(file: &FileDescriptorProto, config: &RendererConfig) -> String {
    match &file.package {
        None => String::new(),
        Some(package) => {
            let mut type_path = TypePath::from_package(package);
            type_path.set_separator(&config.package_separator);
            type_path.set_package_case(Some(config.case_config.package));
            type_path.to_string()
        }
    }
}

fn imports(file: &FileDescriptorProto, ignored_imports: &[String]) -> Result<Vec<ImportContext>> {
    let mut imports = Vec::new();
    for import in &file.dependency {
        if ignored_imports.contains(import) {
            continue;
        }
        imports.push(ImportContext::new(import)?);
    }
    Ok(imports)
}

fn enums(file: &FileDescriptorProto, config: &RendererConfig) -> Result<Vec<EnumContext>> {
    let mut enums = Vec::new();
    for proto in &file.enum_type {
        enums.push(EnumContext::new(proto, config)?);
    }
    Ok(enums)
}

fn messages(
    file: &FileDescriptorProto,
    package: Option<&String>,
    config: &RendererConfig,
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
    insert_builtin_file_options(&mut map, options)
        .map_err(|err| S::Error::custom(file_options_error(err)))?;
    insert_custom_options(&mut map, options, &proto_options::FILE_KEY_VALUE)
        .map_err(|err| S::Error::custom(err.to_string()))?;
    debug!("Serializing file options: {:?}", map);
    serializer.collect_map(map)
}

fn file_options_error(err: impl Error) -> String {
    format!("error in serialize_file_options: {}", err)
}

fn insert_builtin_file_options(
    map: &mut HashMap<String, serde_json::Value>,
    options: &FileOptions,
) -> Result<(), serde_json::Error> {
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
    use crate::renderer::case::Case;
    use anyhow::Result;
    use prost::{Extendable, ExtensionSet};
    use prost_types::{FileDescriptorProto, FileOptions};

    use crate::renderer::context::FileContext;
    use crate::renderer::renderer_config::CaseConfig;
    use crate::renderer::RendererConfig;

    #[test]
    fn source_file() -> Result<()> {
        let config = RendererConfig::default();
        let name = "file_name".to_owned();
        let file = FileDescriptorProto {
            name: Some(name.clone()),
            ..Default::default()
        };
        let context = FileContext::new(&file, &config)?;
        assert_eq!(context.source_file, name);
        Ok(())
    }

    #[test]
    fn package() -> Result<()> {
        let config = RendererConfig {
            package_separator: "::".to_string(),
            case_config: CaseConfig {
                package: Case::Upper,
                ..Default::default()
            },
            ..Default::default()
        };
        let name = "file_name".to_owned();
        let file = FileDescriptorProto {
            name: Some(name.clone()),
            package: Some("some.package.name".to_owned()),
            ..Default::default()
        };
        let context = FileContext::new(&file, &config)?;
        assert_eq!(context.package, "SOME::PACKAGE::NAME");
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let file = FileDescriptorProto::default();
        let result = FileContext::new(&file, &config);
        assert!(result.is_err());
    }

    #[test]
    #[allow(deprecated)]
    fn file_options() -> Result<()> {
        let config = RendererConfig::default();
        let file = FileDescriptorProto {
            name: Some("file_name".to_owned()),
            options: Some(FileOptions {
                java_package: Some("java_package".to_owned()),
                java_outer_classname: Some("java_outer_classname".to_owned()),
                java_multiple_files: Some(true),
                java_generate_equals_and_hash: None,
                java_string_check_utf8: Some(true),
                optimize_for: Some(1234),
                go_package: Some("go_package".to_owned()),
                cc_generic_services: Some(true),
                java_generic_services: Some(true),
                py_generic_services: Some(true),
                php_generic_services: Some(true),
                deprecated: Some(true),
                cc_enable_arenas: Some(true),
                objc_class_prefix: Some("objc_class_prefix".to_owned()),
                csharp_namespace: Some("csharp_namespace".to_owned()),
                swift_prefix: Some("swift_prefix".to_owned()),
                php_class_prefix: Some("php_class_prefix".to_owned()),
                php_namespace: Some("php_namespace".to_owned()),
                php_metadata_namespace: Some("php_metadata_namespace".to_owned()),
                ruby_package: Some("ruby_package".to_owned()),
                uninterpreted_option: vec![],
                extension_set: ExtensionSet::default(),
            }),
            ..Default::default()
        };
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

    #[test]
    fn key_value_options() -> Result<()> {
        let mut options = FileOptions::default();
        options.set_extension_data(
            &proto_options::FILE_KEY_VALUE,
            vec!["key0=value0".to_owned(), "key1=value1".to_owned()],
        )?;
        let file = FileDescriptorProto {
            name: Some("file_name".to_owned()),
            options: Some(options),
            ..Default::default()
        };

        let context = FileContext::new(&file, &RendererConfig::default())?;
        let json = serde_json::to_string(&context)?;
        println!("{}", json);
        assert!(json.contains(r#""key0":"value0""#));
        assert!(json.contains(r#""key1":"value1""#));
        Ok(())
    }

    #[test]
    fn ignored_imports() -> Result<()> {
        let ignored_file = "some/ignored/file.proto";
        let other_file = "included/file.proto";
        let mut config = RendererConfig::default();
        config
            .ignored_imports
            .push("some/ignored/file.proto".to_owned());
        let file = FileDescriptorProto {
            name: Some("name".to_owned()),
            dependency: vec![ignored_file.to_owned(), other_file.to_owned()],
            ..Default::default()
        };

        let context = FileContext::new(&file, &config)?;
        assert_eq!(
            context.imports.len(),
            1,
            "If 2 imports we didn't ignore the configured one"
        );
        assert_eq!(context.imports[0].file_path(), other_file);
        Ok(())
    }
}
