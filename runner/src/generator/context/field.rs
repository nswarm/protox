use crate::generator::primitive;
use crate::generator::template_config::TemplateConfig;
use crate::util;
use anyhow::{anyhow, Result};
use prost_types::FieldDescriptorProto;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
pub struct FieldContext<'a> {
    name: &'a str,
    native_type: &'a str,
}

impl<'a> FieldContext<'a> {
    pub fn new(field: &'a FieldDescriptorProto, config: &'a TemplateConfig) -> Result<Self> {
        let context = Self {
            name: name(field)?,
            native_type: native_type(field, config)?,
        };
        Ok(context)
    }
}

fn name(field: &FieldDescriptorProto) -> Result<&str> {
    util::str_or_error(&field.name, || "Field has no 'name'".to_string())
}

fn native_type<'a>(field: &'a FieldDescriptorProto, config: &'a TemplateConfig) -> Result<&'a str> {
    let native_type = match &field.type_name {
        Some(type_name) => config.type_config.get(type_name).unwrap_or(type_name),
        None => configured_primitive_type_name(field, config)?,
    };
    Ok(native_type)
}

fn configured_primitive_type_name<'a>(
    field: &FieldDescriptorProto,
    config: &'a TemplateConfig,
) -> Result<&'a String> {
    let primitive_name = primitive::from_proto_type(proto_type(field)?)?;
    match config.type_config.get(primitive_name) {
        None => Err(anyhow!(
            "No native type is configured for proto primitive '{}'",
            primitive_name
        )),
        Some(primitive_name) => Ok(primitive_name),
    }
}

fn proto_type(field: &FieldDescriptorProto) -> Result<prost_types::field::Kind> {
    match field.r#type {
        None => Err(anyhow!(
            "Field '{}' has no type.",
            util::str_or_unknown(&field.name)
        )),
        Some(value) => i32_to_proto_type(value),
    }
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

#[cfg(test)]
mod tests {
    use crate::generator::context::field::FieldContext;
    use crate::generator::primitive;
    use crate::generator::template_config::TemplateConfig;
    use anyhow::Result;
    use prost_types::FieldDescriptorProto;

    #[test]
    fn name() -> Result<()> {
        let config = TemplateConfig::default();
        let name = "test_name".to_string();
        let mut field = default_field();
        field.name = Some(name.clone());
        field.type_name = Some(primitive::FLOAT.to_string());
        let context = FieldContext::new(&field, &config)?;
        assert_eq!(context.name.to_string(), name);
        Ok(())
    }

    mod type_name_from_config {
        use crate::generator::context::field::tests::default_field;
        use crate::generator::context::field::FieldContext;
        use crate::generator::template_config::TemplateConfig;
        use anyhow::Result;

        macro_rules! test_type_config {
            ($proto_type:ident) => {
                #[test]
                fn $proto_type() -> Result<()> {
                    test_type_config(stringify!($proto_type))
                }
            };
        }

        test_type_config!(float);
        test_type_config!(double);
        test_type_config!(int32);
        test_type_config!(int64);
        test_type_config!(uint32);
        test_type_config!(uint64);
        test_type_config!(sint32);
        test_type_config!(sint64);
        test_type_config!(fixed32);
        test_type_config!(fixed64);
        test_type_config!(bool);
        test_type_config!(string);
        test_type_config!(bytes);

        fn test_type_config(proto_type_name: &str) -> Result<()> {
            let mut config = TemplateConfig::default();
            config.type_config.insert(
                proto_type_name.to_string(),
                ["TEST", proto_type_name].concat(),
            );
            let mut field = default_field();
            field.name = Some("field_name".to_string());
            field.type_name = Some(proto_type_name.to_string());
            let context = FieldContext::new(&field, &config)?;
            assert_eq!(
                Some(&context.native_type.to_string()),
                config.type_config.get(proto_type_name),
            );
            Ok(())
        }
    }

    #[test]
    fn missing_name_errors() {
        let config = TemplateConfig::default();
        let mut field = default_field();
        field.type_name = Some(primitive::FLOAT.to_string());
        let result = FieldContext::new(&field, &config);
        assert!(result.is_err());
    }

    #[test]
    fn missing_type_name_errors() {
        let config = TemplateConfig::default();
        let mut field = default_field();
        field.name = Some("field_name".to_string());
        let result = FieldContext::new(&field, &config);
        assert!(result.is_err());
    }

    fn default_field() -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: None,
            number: None,
            label: None,
            r#type: None,
            type_name: None,
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }
}
