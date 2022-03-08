use crate::template_renderer::proto::TypePath;
use crate::template_renderer::renderer_config::RendererConfig;
use crate::template_renderer::{primitive, proto};
use crate::util;
use anyhow::{anyhow, Result};
use prost::Extendable;
use prost_types::FieldDescriptorProto;

#[derive(Clone, Debug)]
pub enum ProtoType {
    Type(i32),
    TypeName(String),
    NativeTypeOverride(String),
}

#[derive(Eq, PartialEq)]
enum ChangeCase {
    Yes,
    No,
}

impl ProtoType {
    pub fn from_field(field: &FieldDescriptorProto) -> Result<Self> {
        if let Some(native_type) = native_type_override(field) {
            return Ok(ProtoType::NativeTypeOverride(native_type.to_string()));
        }
        match &field.type_name {
            None => match field.r#type {
                None => Err(error_missing_type(field)),
                Some(proto_type_id) => Ok(ProtoType::Type(proto_type_id)),
            },
            Some(type_name) => Ok(ProtoType::TypeName(type_name.to_string())),
        }
    }

    pub fn to_type_path<'a>(&self, config: &'a RendererConfig) -> Result<TypePath<'a>> {
        let result = match self {
            ProtoType::Type(proto_type) => primitive_type_path(*proto_type, config)?,
            ProtoType::TypeName(type_name) => {
                complex_type_path(&type_name, config, ChangeCase::Yes)
            }
            ProtoType::NativeTypeOverride(type_name) => {
                complex_type_path(&type_name, config, ChangeCase::No)
            }
        };
        Ok(result)
    }
}

fn native_type_override(field: &FieldDescriptorProto) -> Option<&str> {
    match field.options.as_ref() {
        None => None,
        Some(options) => options
            .extension_data(proto_options::NATIVE_TYPE)
            .map(&String::as_str)
            .ok(),
    }
}

fn primitive_type_path(proto_type_id: i32, config: &RendererConfig) -> Result<TypePath> {
    let primitive_type_name = primitive_type_name(proto_type_id, config)?;
    Ok(proto::TypePath::from_type(primitive_type_name))
}

fn complex_type_path<'a>(
    type_name: &str,
    config: &'a RendererConfig,
    change_case: ChangeCase,
) -> TypePath<'a> {
    let type_name = complex_type_name(&type_name, config);
    let mut type_path = proto::TypePath::from_type(type_name);
    if change_case == ChangeCase::Yes {
        type_path.set_name_case(Some(config.case_config.message_name));
    }
    type_path.set_package_case(Some(config.case_config.import));
    type_path.set_separator(&config.package_separator);
    type_path
}

pub fn primitive_type_name(proto_type_id: i32, config: &RendererConfig) -> Result<&str> {
    let primitive_name = primitive::from_proto_type(i32_to_proto_type(proto_type_id)?)?;
    match config.type_config.get(primitive_name) {
        None => Err(anyhow!(
            "No native type is configured for proto primitive '{}'",
            primitive_name
        )),
        Some(primitive_name) => Ok(primitive_name),
    }
}

fn complex_type_name<'a>(type_name: &'a str, config: &'a RendererConfig) -> &'a str {
    let type_name = proto::normalize_prefix(type_name);
    let type_name = config
        .type_config
        .get(type_name)
        .map(String::as_str)
        .unwrap_or(type_name);
    type_name
}

fn i32_to_proto_type(val: i32) -> Result<prost_types::field::Kind> {
    match val {
        1 => Ok(prost_types::field::Kind::TypeDouble),
        2 => Ok(prost_types::field::Kind::TypeFloat),
        3 => Ok(prost_types::field::Kind::TypeInt64),
        4 => Ok(prost_types::field::Kind::TypeUint64),
        5 => Ok(prost_types::field::Kind::TypeInt32),
        6 => Ok(prost_types::field::Kind::TypeFixed64),
        7 => Ok(prost_types::field::Kind::TypeFixed32),
        8 => Ok(prost_types::field::Kind::TypeBool),
        9 => Ok(prost_types::field::Kind::TypeString),
        10 => Ok(prost_types::field::Kind::TypeGroup),
        11 => Ok(prost_types::field::Kind::TypeMessage),
        12 => Ok(prost_types::field::Kind::TypeBytes),
        13 => Ok(prost_types::field::Kind::TypeUint32),
        14 => Ok(prost_types::field::Kind::TypeEnum),
        15 => Ok(prost_types::field::Kind::TypeSfixed32),
        16 => Ok(prost_types::field::Kind::TypeSfixed64),
        17 => Ok(prost_types::field::Kind::TypeSint32),
        18 => Ok(prost_types::field::Kind::TypeSint64),
        _ => Err(anyhow!("i32 '{}' does not map to a valid proto type.", val)),
    }
}

fn error_missing_type(field: &FieldDescriptorProto) -> anyhow::Error {
    anyhow!(
        "Field '{}' has no type and cannot be viewed as a ProtoType",
        util::str_or_unknown(&field.name)
    )
}
