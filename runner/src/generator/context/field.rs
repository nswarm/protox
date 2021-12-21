use crate::generator::context::util;
use crate::generator::template_config::TemplateConfig;
use anyhow::{anyhow, Result};
use prost_types::FieldDescriptorProto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FieldContext<'a> {
    name: &'a str,
    type_name: &'a str,
}

impl<'a> FieldContext<'a> {
    pub fn new(field: &'a FieldDescriptorProto, config: &'a TemplateConfig) -> Result<Self> {
        let context = Self {
            name: name(field)?,
            type_name: type_name(field, config)?,
        };
        Ok(context)
    }
}

fn name(field: &FieldDescriptorProto) -> Result<&str> {
    util::str_or_error(&field.name, || "Field has no 'name'".to_string())
}

fn type_name<'a>(field: &'a FieldDescriptorProto, config: &'a TemplateConfig) -> Result<&'a str> {
    let type_name = util::str_or_error(&field.type_name, || {
        format!(
            "Field has no 'type name': {:?}",
            field.name.as_ref().unwrap_or(&"(unknown)".to_string())
        )
    })?;
    let type_name = config
        .type_config
        .get(type_name)
        .ok_or(anyhow!("Invalid primitive type name {}", type_name))?;
    Ok(type_name)
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
                Some(&context.type_name.to_string()),
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
