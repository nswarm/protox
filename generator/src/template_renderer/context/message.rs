use crate::template_renderer::case::Case;
use crate::template_renderer::context::proto_type::ProtoType;
use crate::template_renderer::context::FieldContext;
use crate::template_renderer::option_key_value::insert_custom_options;
use crate::template_renderer::proto::PACKAGE_SEPARATOR;
use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;
use anyhow::{anyhow, Context, Result};
use log::debug;
use prost_types::{DescriptorProto, FieldDescriptorProto, MessageOptions};
use serde::ser::Error;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct MessageContext {
    /// Name of this message.
    name: String,

    /// Fields available in this message.
    fields: Vec<FieldContext>,

    /// Proto message options are serialized as an object like so:
    /// ```json
    /// {
    ///   "options": {
    ///       "option_name": <option_value>,
    ///   }
    ///   ...etc.
    /// }
    /// ```
    /// Which can be accessed in the template like `{{options.option_name}}`. Options which have no
    /// value will not exist in the context, so you probably want to if guard:
    /// ```handlebars
    /// {{#if options.option_name}}
    ///   {{options.option_name}}
    /// {{/if}}
    /// ```
    /// Note that for boolean values one #if is enough to check both that it exists and is true.
    ///
    /// Built-in proto option names and types can be seen here:
    /// https://docs.rs/prost-types/latest/prost_types/struct.MessageOptions.html
    ///
    /// Additionally, a few idlx-specific options are supported. See the proto files at
    /// `idlx/proto_options/protos` for more info.
    #[serde(serialize_with = "serialize_message_options", skip_deserializing)]
    options: Option<MessageOptions>,
}

impl MessageContext {
    pub fn new(
        message: &DescriptorProto,
        package: Option<&String>,
        config: &RendererConfig,
    ) -> Result<Self> {
        log_new_message(&message.name);
        let context = Self {
            name: name(message, config.case_config.message_name)?,
            fields: fields(message, package, config)?,
            options: message.options.clone(),
        };
        Ok(context)
    }
}

pub type MapData = HashMap<String, MapEntryData>;
pub struct MapEntryData {
    pub key: ProtoType,
    pub value: ProtoType,
}

fn log_new_message(name: &Option<String>) {
    debug!("Creating message context: {}", util::str_or_unknown(name));
}

fn name(message: &DescriptorProto, case: Case) -> Result<String> {
    let name = util::str_or_error(&message.name, || "Message has no 'name'".to_string())?;
    Ok(case.rename(name))
}

fn fields(
    message: &DescriptorProto,
    package: Option<&String>,
    config: &RendererConfig,
) -> Result<Vec<FieldContext>> {
    let map_data = collect_map_data(message, package)?;
    let mut fields = Vec::new();
    for field in &message.field {
        fields.push(FieldContext::new(field, package, &map_data, config)?);
    }
    Ok(fields)
}

fn collect_map_data(message: &DescriptorProto, package: Option<&String>) -> Result<MapData> {
    let message_name = util::str_or_error(&message.name, || {
        "collect_map_data: No message name.".to_string()
    })?;
    let mut map_data = MapData::new();
    for nested in message.nested_type.iter().filter(is_map) {
        let (key, value) = find_map_key_value(nested, message_name)?;
        let fully_qualified_nested_type =
            fully_qualify_map_type(&nested_name(&nested, message_name)?, message_name, package);
        map_data.insert(fully_qualified_nested_type, MapEntryData { key, value });
    }
    Ok(map_data)
}

fn find_map_key_value(
    nested: &DescriptorProto,
    outer_msg_name: &str,
) -> Result<(ProtoType, ProtoType)> {
    static KEY_FIELD_NAME: &str = "key";
    static VALUE_FIELD_NAME: &str = "value";
    let key = find_field_type(KEY_FIELD_NAME, &nested.field)
        .with_context(|| error_context_failed_collect_map_data(outer_msg_name, &nested.name))?;
    let value = find_field_type(VALUE_FIELD_NAME, &nested.field)
        .with_context(|| error_context_failed_collect_map_data(outer_msg_name, &nested.name))?;
    Ok((key, value))
}

fn find_field_type(field_name: &str, fields: &[FieldDescriptorProto]) -> Result<ProtoType> {
    for field in fields {
        if let Some(name) = &field.name {
            if name == field_name {
                return ProtoType::from_field(field);
            }
        }
    }
    Err(anyhow!(
        "Could not find required field name: {}.",
        field_name
    ))
}

fn is_map(message: &&DescriptorProto) -> bool {
    match &message.options {
        None => false,
        Some(options) => match options.map_entry {
            None => false,
            Some(is_map) => is_map,
        },
    }
}

fn nested_name(nested: &DescriptorProto, message_name: &str) -> Result<String> {
    nested.name.clone().ok_or(anyhow!(
        "Nested message has no name, outer message: {}",
        message_name
    ))
}

fn fully_qualify_map_type(entry_type: &str, outer_type: &str, package: Option<&String>) -> String {
    let mut fully_qualified = String::new();
    // All fully-qualified proto paths start with a separator.
    fully_qualified.push(PACKAGE_SEPARATOR);
    if let Some(package) = package {
        fully_qualified.push_str(package);
        fully_qualified.push(PACKAGE_SEPARATOR);
    }
    fully_qualified.push_str(outer_type);
    fully_qualified.push(PACKAGE_SEPARATOR);
    fully_qualified.push_str(entry_type);
    fully_qualified
}

fn error_context_failed_collect_map_data(
    message_name: &str,
    nested_name: &Option<String>,
) -> String {
    format!(
        "collect_map_data - Message '{}', nested message '{}'",
        message_name,
        util::str_or_unknown(&nested_name)
    )
}

fn serialize_message_options<S: Serializer>(
    options: &Option<MessageOptions>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let options = match options {
        None => return serializer.serialize_none(),
        Some(options) => options,
    };
    let mut map = HashMap::new();
    insert_custom_options(&mut map, options, &proto_options::MSG_KEY_VALUE)
        .map_err(|err| S::Error::custom(err.to_string()))?;
    debug!("Serializing message options: {:?}", map);
    serializer.collect_map(map)
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::case::Case;
    use crate::template_renderer::context::message::MessageContext;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::{DescriptorProto, FieldDescriptorProto, MessageOptions};

    #[test]
    fn name() -> Result<()> {
        let config = RendererConfig::default();
        let msg_name = "MsgName".to_string();
        let mut message = default_message();
        message.name = Some(msg_name.clone());
        let context = MessageContext::new(&message, None, &config)?;
        assert_eq!(context.name, msg_name);
        Ok(())
    }

    #[test]
    fn name_with_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.message_name = Case::UpperSnake;
        let msg_name = "msgName".to_string();
        let mut message = default_message();
        message.name = Some(msg_name.clone());
        let context = MessageContext::new(&message, None, &config)?;
        assert_eq!(context.name, "MSG_NAME");
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

    #[test]
    fn key_value_options() -> Result<()> {
        let config = RendererConfig::default();
        let mut message = default_message();
        message.name = Some("MessageName".to_string());
        let mut options = MessageOptions::default();
        options.set_extension_data(
            &proto_options::MSG_KEY_VALUE,
            vec!["key0=value0".to_string(), "key1=value1".to_string()],
        )?;
        message.options = Some(options);

        let context = MessageContext::new(&message, None, &config)?;
        let json = serde_json::to_string(&context)?;
        println!("{}", json);
        assert!(json.contains(r#""key0":"value0""#));
        assert!(json.contains(r#""key1":"value1""#));
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
