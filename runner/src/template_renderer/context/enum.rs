use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;
use anyhow::{anyhow, Result};
use log::debug;
use prost_types::EnumDescriptorProto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EnumContext<'a> {
    name: &'a str,
    values: Vec<EnumValueContext>,
}

#[derive(Serialize, Deserialize)]
pub struct EnumValueContext {
    name: String,
    number: i32,
}

impl<'a> EnumContext<'a> {
    pub fn new(proto: &'a EnumDescriptorProto, _config: &'a RendererConfig) -> Result<Self> {
        log_new_enum(&proto.name);
        let context = Self {
            name: name(&proto)?,
            values: values(&proto)?,
        };
        Ok(context)
    }
}

fn log_new_enum(name: &Option<String>) {
    debug!("Creating message context: {}", util::str_or_unknown(name));
}

fn name(proto: &EnumDescriptorProto) -> Result<&str> {
    util::str_or_error(&proto.name, || "Enum has no 'name'".to_string())
}

fn values(proto: &EnumDescriptorProto) -> Result<Vec<EnumValueContext>> {
    let mut values = Vec::new();
    for value in &proto.value {
        let (name, number) = match (value.name.clone(), value.number) {
            (Some(name), Some(number)) => (name, number),
            _ => return Err(error_invalid_value(&value.name)),
        };
        values.push(EnumValueContext { name, number });
    }
    Ok(values)
}

fn error_invalid_value(name: &Option<String>) -> anyhow::Error {
    anyhow!(
        "Enum '{}' has a value missing name and/or number.",
        util::str_or_unknown(name),
    )
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::context::EnumContext;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;
    use prost_types::{EnumDescriptorProto, EnumValueDescriptorProto};

    #[test]
    fn name() -> Result<()> {
        let config = RendererConfig::default();
        let enum_name = "msg_name".to_string();
        let mut proto = default_enum();
        proto.name = Some(enum_name.clone());
        let context = EnumContext::new(&proto, &config)?;
        assert_eq!(context.name, enum_name);
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let proto = default_enum();
        let result = EnumContext::new(&proto, &config);
        assert!(result.is_err());
    }

    #[test]
    fn creates_values_from_proto() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = default_enum();
        proto.name = Some("enum_name".to_string());
        proto.value.push(enum_value(1));
        proto.value.push(enum_value(2));
        let context = EnumContext::new(&proto, &config)?;
        assert_eq!(context.values[0].name, "1");
        assert_eq!(context.values[0].number, 1);
        assert_eq!(context.values[1].name, "2");
        assert_eq!(context.values[1].number, 2);
        Ok(())
    }

    fn enum_value(number: i32) -> EnumValueDescriptorProto {
        EnumValueDescriptorProto {
            name: Some(number.to_string()),
            number: Some(number),
            options: None,
        }
    }

    fn default_enum() -> EnumDescriptorProto {
        EnumDescriptorProto {
            name: None,
            value: vec![],
            options: None,
            reserved_range: vec![],
            reserved_name: vec![],
        }
    }
}
