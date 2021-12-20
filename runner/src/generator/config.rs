use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub type_config: TypeConfig,
}

/// Primitive type mapping for proto -> lang.
/// https://developers.google.com/protocol-buffers/docs/proto3#scalar
#[derive(Serialize, Deserialize, Default)]
pub struct TypeConfig {
    pub float32: String,
    pub float64: String,
    pub int32: String,
    pub int64: String,
    pub uint32: String,
    pub uint64: String,
    pub sint32: String,
    pub sint64: String,
    pub fixed32: String,
    pub fixed64: String,
    pub bool: String,
    pub string: String,
    pub bytes: String,
}
