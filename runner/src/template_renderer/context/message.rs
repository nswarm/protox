use crate::template_renderer::context::FieldContext;
use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;
use anyhow::Result;
use log::debug;
use prost_types::DescriptorProto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageContext<'a> {
    name: &'a str,
    fields: Vec<FieldContext<'a>>,
}

impl<'a> MessageContext<'a> {
    pub fn new(
        message: &'a DescriptorProto,
        package: Option<&String>,
        config: &'a RendererConfig,
    ) -> Result<Self> {
        log_new_message(&message.name);
        let context = Self {
            name: name(message)?,
            fields: fields(message, package, config)?,
        };
        Ok(context)
    }
}

fn log_new_message(name: &Option<String>) {
    debug!("Creating message context: {}", util::str_or_unknown(name));
}

fn name(message: &DescriptorProto) -> Result<&str> {
    util::str_or_error(&message.name, || "Message has no 'name'".to_string())
}

fn fields<'a>(
    message: &'a DescriptorProto,
    package: Option<&String>,
    config: &'a RendererConfig,
) -> Result<Vec<FieldContext<'a>>> {
    let mut fields = Vec::new();
    for field in &message.field {
        fields.push(FieldContext::new(field, package, config)?);
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::context::message::MessageContext;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;
    use prost_types::{DescriptorProto, FieldDescriptorProto};

    #[test]
    fn name() -> Result<()> {
        let config = RendererConfig::default();
        let msg_name = "msg_name".to_string();
        let mut message = default_message();
        message.name = Some(msg_name.clone());
        let context = MessageContext::new(&message, None, &config)?;
        assert_eq!(context.name, msg_name);
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let message = default_message();
        let result = MessageContext::new(&message, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn creates_fields_from_proto() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = default_message();
        proto.name = Some("enum_name".to_string());
        proto.field.push(field("field0"));
        proto.field.push(field("field1"));
        let context = MessageContext::new(&proto, None, &config)?;
        assert_eq!(context.fields.get(0).map(|f| f.name()), Some("field0"));
        assert_eq!(context.fields.get(1).map(|f| f.name()), Some("field1"));
        Ok(())
    }

    fn field(name: impl ToString) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.to_string()),
            number: None,
            label: None,
            r#type: None,
            type_name: Some("type".to_string()),
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }

    fn default_message() -> DescriptorProto {
        DescriptorProto {
            name: None,
            field: vec![],
            extension: vec![],
            nested_type: vec![],
            enum_type: vec![],
            extension_range: vec![],
            oneof_decl: vec![],
            options: None,
            reserved_range: vec![],
            reserved_name: vec![],
        }
    }
}
