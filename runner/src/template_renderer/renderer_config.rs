use crate::template_renderer::primitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct RendererConfig {
    /// The file extension to use for generated files.
    pub file_extension: String,

    /// Defines the primitive type mapping for proto -> lang.
    /// https://developers.google.com/protocol-buffers/docs/proto3#scalar
    ///
    /// Each primitive::* type should have a value that will be used in templates.
    /// e.g. { "int64": "i64", "int32": "i32", ...etc }
    pub type_config: HashMap<String, String>,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            file_extension: "".to_string(),
            type_config: default_type_config(),
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
