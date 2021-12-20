use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Defines the primitive type mapping for proto -> lang.
    /// https://developers.google.com/protocol-buffers/docs/proto3#scalar
    ///
    /// Each of these types should have a value that will be used in templates for that proto type.
    /// float
    /// double
    /// int32
    /// int64
    /// uint32
    /// uint64
    /// sint32
    /// sint64
    /// fixed32
    /// fixed64
    /// bool
    /// string
    /// bytes
    ///
    /// e.g. { "int64": "i64", "int32": "i32", ...etc }
    pub type_config: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            type_config: default_type_config(),
        }
    }
}

fn default_type_config() -> HashMap<String, String> {
    let mut type_config = HashMap::new();
    type_config.insert("float".into(), "float".into());
    type_config.insert("double".into(), "double".into());
    type_config.insert("int32".into(), "int32".into());
    type_config.insert("int64".into(), "int64".into());
    type_config.insert("uint32".into(), "uint32".into());
    type_config.insert("uint64".into(), "uint64".into());
    type_config.insert("sint32".into(), "sint32".into());
    type_config.insert("sint64".into(), "sint64".into());
    type_config.insert("fixed32".into(), "fixed32".into());
    type_config.insert("fixed64".into(), "fixed64".into());
    type_config.insert("bool".into(), "bool".into());
    type_config.insert("string".into(), "string".into());
    type_config.insert("bytes".into(), "bytes".into());
    type_config
}
