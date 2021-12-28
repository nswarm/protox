use anyhow::{anyhow, Result};
use log::debug;
use prost_types::FieldDescriptorProto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::template_renderer::renderer_config::RendererConfig;
use crate::template_renderer::{primitive, proto};
use crate::util;

#[derive(Serialize, Deserialize)]
pub struct FieldContext<'a> {
    field_name: &'a str,
    /// Type as defined by type config or literal type name.
    ///
    /// ```txt
    ///      pkg.sub_pkg.TypeName
    /// ```
    fully_qualified_type: String,
    /// Type relative to the owning file's package.
    ///
    /// ```txt
    ///      package:  pkg.sub
    ///      type:     pkg.sub.deep.TypeName
    ///      relative: deep.TypeName
    /// ```
    relative_type: String,
}

impl<'a> FieldContext<'a> {
    pub fn new(
        field: &'a FieldDescriptorProto,
        package: Option<&String>,
        config: &'a RendererConfig,
    ) -> Result<Self> {
        log_new_field(&field.name);
        let fully_qualified_type = fully_qualified_type(field, config)?;
        let mut proto_type = proto::TypePath::from_type(fully_qualified_type);
        proto_type.set_separator(&config.package_separator);
        let context = Self {
            field_name: field_name(field, &config.field_name_override)?,
            fully_qualified_type: proto_type.to_string(),
            relative_type: proto_type
                .relative_to(package, config.field_relative_parent_prefix.as_ref()),
        };
        Ok(context)
    }
}

fn log_new_field(name: &Option<String>) {
    debug!("Creating field context: {}", util::str_or_unknown(name));
}

fn field_name<'a>(
    field: &'a FieldDescriptorProto,
    overrides: &'a HashMap<String, String>,
) -> Result<&'a str> {
    let field_name = util::str_or_error(&field.name, || "Field has no 'name'".to_string())?;
    Ok(overrides
        .get(field_name)
        .map(String::as_str)
        .unwrap_or(field_name))
}

fn fully_qualified_type<'a>(
    field: &'a FieldDescriptorProto,
    config: &'a RendererConfig,
) -> Result<&'a str> {
    let fully_qualified_type = match &field.type_name {
        Some(type_name) => {
            let type_name = proto::normalize_prefix(type_name);
            config
                .type_config
                .get(type_name)
                .map(String::as_str)
                .unwrap_or(type_name)
        }
        None => configured_primitive_type_name(field, config)?,
    };
    Ok(fully_qualified_type)
}

fn configured_primitive_type_name<'a>(
    field: &FieldDescriptorProto,
    config: &'a RendererConfig,
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
    use anyhow::Result;
    use prost_types::FieldDescriptorProto;

    use crate::template_renderer::context::field::FieldContext;
    use crate::template_renderer::primitive;
    use crate::template_renderer::renderer_config::RendererConfig;

    #[test]
    fn field_name() -> Result<()> {
        let config = RendererConfig::default();
        let name = "test_name".to_string();
        let mut field = default_field();
        field.name = Some(name.clone());
        field.type_name = Some(primitive::FLOAT.to_string());
        let context = FieldContext::new(&field, None, &config)?;
        assert_eq!(context.field_name.to_string(), name);
        Ok(())
    }

    #[test]
    fn override_field_name() -> Result<()> {
        let old_name = "old_name".to_string();
        let new_name = "new_name".to_string();
        let mut config = RendererConfig::default();
        config
            .field_name_override
            .insert(old_name.clone(), new_name.clone());
        let mut field = default_field();
        field.name = Some(old_name);
        field.type_name = Some(primitive::FLOAT.to_string());
        let context = FieldContext::new(&field, None, &config)?;
        assert_eq!(context.field_name.to_string(), new_name);
        Ok(())
    }

    mod type_name_from_config {
        use anyhow::Result;

        use crate::template_renderer::context::field::tests::default_field;
        use crate::template_renderer::context::field::FieldContext;
        use crate::template_renderer::renderer_config::RendererConfig;

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
            let mut config = RendererConfig::default();
            config.type_config.insert(
                proto_type_name.to_string(),
                ["TEST", proto_type_name].concat(),
            );
            let mut field = default_field();
            field.name = Some("field_name".to_string());
            field.type_name = Some(proto_type_name.to_string());
            let context = FieldContext::new(&field, None, &config)?;
            assert_eq!(
                Some(&context.fully_qualified_type.to_string()),
                config.type_config.get(proto_type_name),
            );
            Ok(())
        }
    }

    #[test]
    fn package_separator_replaced_in_types() -> Result<()> {
        let mut field = default_field();
        field.name = Some("test".to_string());
        field.type_name = Some(".root.sub.TypeName".to_string());
        let mut config = RendererConfig::default();
        config.package_separator = "::".to_string();
        let context = FieldContext::new(&field, Some(&"root".to_string()), &config)?;
        assert_eq!(context.relative_type, "sub::TypeName");
        assert_eq!(context.fully_qualified_type, "root::sub::TypeName");
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let mut field = default_field();
        field.type_name = Some(primitive::FLOAT.to_string());
        let result = FieldContext::new(&field, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn missing_type_name_errors() {
        let config = RendererConfig::default();
        let mut field = default_field();
        field.name = Some("field_name".to_string());
        let result = FieldContext::new(&field, None, &config);
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
