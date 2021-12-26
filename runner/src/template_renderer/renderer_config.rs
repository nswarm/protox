use crate::template_renderer::primitive;
use crate::template_renderer::renderer::Renderer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct RendererConfig {
    /// The file extension to use for generated files.
    pub file_extension: String,

    /// Defines the type mapping for proto -> lang.
    /// https://developers.google.com/protocol-buffers/docs/proto3#scalar
    ///
    /// Each primitive::* type MUST have a value specified that will be used in templates.
    /// e.g. { "int64": "i64", "int32": "i32", ...etc }
    ///
    /// This can also be used to override types. You need to specify the fully-qualified type name.
    /// e.g. { "root.sub.TypeName": "MyFancyType" }
    pub type_config: HashMap<String, String>,

    /// Name of directory metadata files.
    /// default: "metadata"
    #[serde(default = "default_metadata_file_name")]
    pub metadata_file_name: String,

    /// Separator used in qualified type names.
    /// e.g. root.sub.TypeName
    ///          ^   ^
    /// default: `.`
    #[serde(default = "default_package_separator")]
    pub package_separator: String,

    /// Override field names declared by the proto, for example when a proto uses a keyword as a
    /// field name in your target language.
    /// e.g. { "enum": "new_name" }
    /// Would replace any fields called `enum` with `new_name`.
    #[serde(default)]
    pub field_name_override: HashMap<String, String>,
}

fn default_metadata_file_name() -> String {
    Renderer::METADATA_TEMPLATE_NAME.to_string()
}

fn default_package_separator() -> String {
    ".".to_string()
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            file_extension: "".to_string(),
            type_config: default_type_config(),
            metadata_file_name: default_metadata_file_name(),
            package_separator: default_package_separator(),
            field_name_override: Default::default(),
        }
    }
}

fn default_type_config() -> HashMap<String, String> {
    let mut type_config = HashMap::new();
    type_config.insert(primitive::FLOAT.into(), primitive::FLOAT.into());
    type_config.insert(primitive::DOUBLE.into(), primitive::DOUBLE.into());
    type_config.insert(primitive::INT32.into(), primitive::INT32.into());
    type_config.insert(primitive::INT64.into(), primitive::INT64.into());
    type_config.insert(primitive::UINT32.into(), primitive::UINT32.into());
    type_config.insert(primitive::UINT64.into(), primitive::UINT64.into());
    type_config.insert(primitive::SINT32.into(), primitive::SINT32.into());
    type_config.insert(primitive::SINT64.into(), primitive::SINT64.into());
    type_config.insert(primitive::FIXED32.into(), primitive::FIXED32.into());
    type_config.insert(primitive::FIXED64.into(), primitive::FIXED64.into());
    type_config.insert(primitive::BOOL.into(), primitive::BOOL.into());
    type_config.insert(primitive::STRING.into(), primitive::STRING.into());
    type_config.insert(primitive::BYTES.into(), primitive::BYTES.into());
    type_config
}
