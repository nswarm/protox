use anyhow::{anyhow, Result};
use prost_types::field::Kind;

pub const FLOAT: &str = "float";
pub const DOUBLE: &str = "double";
pub const INT32: &str = "int32";
pub const INT64: &str = "int64";
pub const UINT32: &str = "uint32";
pub const UINT64: &str = "uint64";
pub const SINT32: &str = "sint32";
pub const SINT64: &str = "sint64";
pub const FIXED32: &str = "fixed32";
pub const FIXED64: &str = "fixed64";
pub const SFIXED32: &str = "sfixed32";
pub const SFIXED64: &str = "sfixed64";
pub const BOOL: &str = "bool";
pub const STRING: &str = "string";
pub const BYTES: &str = "bytes";

pub fn from_proto_type(kind: prost_types::field::Kind) -> Result<&'static str> {
    match kind {
        Kind::TypeDouble => Ok(DOUBLE),
        Kind::TypeFloat => Ok(FLOAT),
        Kind::TypeInt64 => Ok(INT64),
        Kind::TypeUint64 => Ok(UINT64),
        Kind::TypeInt32 => Ok(INT32),
        Kind::TypeFixed64 => Ok(FIXED64),
        Kind::TypeFixed32 => Ok(FIXED32),
        Kind::TypeBool => Ok(BOOL),
        Kind::TypeString => Ok(STRING),
        Kind::TypeBytes => Ok(BYTES),
        Kind::TypeUint32 => Ok(UINT32),
        Kind::TypeSfixed32 => Ok(SFIXED32),
        Kind::TypeSfixed64 => Ok(SFIXED64),
        Kind::TypeSint32 => Ok(SINT32),
        Kind::TypeSint64 => Ok(SINT64),
        Kind::TypeGroup => Err(anyhow!("'Group' type is not a primitive type.")),
        Kind::TypeMessage => Err(anyhow!("'Message' type is not a primitive type.")),
        Kind::TypeEnum => Err(anyhow!("'Enum' type is not a primitive type.")),
        Kind::TypeUnknown => Err(anyhow!("Unknown primitive type: {:?}", kind)),
    }
}
