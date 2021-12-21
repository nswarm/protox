use crate::generator::config::Config;
use crate::generator::context::{util, RenderedField};
use anyhow::Result;
use prost_types::DescriptorProto;
use serde::{Deserialize, Serialize};

pub fn message_context_json(
    message: &DescriptorProto,
    config: &Config,
) -> Result<serde_json::Value> {
    let context = MessageContext::new(message, config)?;
    Ok(serde_json::to_value(&context)?)
}

#[derive(Serialize, Deserialize)]
pub struct MessageContext<'a> {
    name: &'a str,

    // Must be rendered and supplied externally.
    pub fields: Vec<RenderedField>,
}

impl<'a> MessageContext<'a> {
    pub fn new(message: &'a DescriptorProto, config: &Config) -> Result<Self> {
        let context = Self {
            name: name(message)?,
            fields: Vec::new(),
        };
        Ok(context)
    }
}

fn name(message: &DescriptorProto) -> Result<&str> {
    util::str_or_error(&message.name, || "Message has no 'name'".to_string())
}

#[cfg(test)]
mod tests {
    use crate::generator::config::Config;
    use crate::generator::context::message::MessageContext;
    use anyhow::Result;
    use prost_types::{DescriptorProto, FieldDescriptorProto};

    #[test]
    fn name() -> Result<()> {
        let config = Config::default();
        let msg_name = "msg_name".to_string();
        let message = message_with_name(msg_name.clone(), vec![]);
        let context = MessageContext::new(&message, &config)?;
        assert_eq!(context.name, msg_name);
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = Config::default();
        let message = DescriptorProto {
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
        };
        let result = MessageContext::new(&message, &config);
        assert!(result.is_err());
    }

    fn message_with_name(name: String, fields: Vec<FieldDescriptorProto>) -> DescriptorProto {
        DescriptorProto {
            name: Some(name),
            field: fields,
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
