use crate::generator::config::Config;
use anyhow::{anyhow, Result};
use prost_types::FieldDescriptorProto;
use serde::{Deserialize, Serialize};

pub fn field(field: &FieldDescriptorProto, config: &Config) -> Result<serde_json::Value> {
    let context = FieldContext::new(field, config)?;
    Ok(serde_json::to_value(&context)?)
}

#[derive(Serialize, Deserialize)]
struct FieldContext<'a> {
    name: &'a str,
    type_name: &'a str,
}

impl<'a> FieldContext<'a> {
    fn new(field: &'a FieldDescriptorProto, config: &'a Config) -> Result<Self> {
        let context = Self {
            name: name(field)?,
            type_name: type_name(field, config)?,
        };
        Ok(context)
    }
}

fn name(field: &FieldDescriptorProto) -> Result<&str> {
    str_or_error(&field.name, || "Field has no 'name'".to_string())
}

fn type_name<'a>(field: &'a FieldDescriptorProto, config: &'a Config) -> Result<&'a str> {
    let type_name = str_or_error(&field.type_name, || {
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

fn str_or_error<F: Fn() -> String>(value: &Option<String>, error: F) -> Result<&str> {
    let result = value
        .as_ref()
        .map(String::as_str)
        .ok_or(anyhow!("{}", error()))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::generator::config::Config;
    use crate::generator::context::FieldContext;
    use anyhow::Result;
    use prost_types::FieldDescriptorProto;

    #[test]
    fn name_to_json() -> Result<()> {
        let mut config = Config::default();
        let name = "test_name".to_string();
        let field = field_with_name(name.clone());
        let context = FieldContext::new(&field, &config)?;
        assert_eq!(context.name.to_string(), name);
        Ok(())
    }

    mod type_name_from_config {
        use crate::generator::config::Config;
        use crate::generator::context::tests::field_with_type;
        use crate::generator::context::FieldContext;
        use anyhow::Result;
        use prost_types::FieldDescriptorProto;

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
            let mut config = Config::default();
            config.type_config.insert(
                proto_type_name.to_string(),
                ["TEST", proto_type_name].concat(),
            );
            let field = field_with_type(proto_type_name.to_string());
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
        let mut config = Config::default();
        let mut field = empty_field();
        field.type_name = Some("float".to_string());
        let result = FieldContext::new(&field, &config);
        assert!(result.is_err());
    }

    #[test]
    fn missing_type_name_errors() {
        let mut config = Config::default();
        let mut field = empty_field();
        field.name = Some("field_name".to_string());
        let result = FieldContext::new(&field, &config);
        assert!(result.is_err());
    }

    fn empty_field() -> FieldDescriptorProto {
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

    fn field_with_name(name: String) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name),
            number: None,
            label: None,
            r#type: None,
            type_name: Some("float".to_string()),
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }

    fn field_with_type(proto_type: String) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some("field_name".to_string()),
            number: None,
            label: None,
            r#type: None,
            type_name: Some(proto_type),
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }
}
