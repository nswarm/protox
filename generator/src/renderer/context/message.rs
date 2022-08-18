use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use log::debug;
use prost_types::{DescriptorProto, FieldDescriptorProto, MessageOptions};
use serde::{Deserialize, Serialize, Serializer};

use crate::renderer::case::Case;
use crate::renderer::context::overlayed::Overlayed;
use crate::renderer::context::proto_type::ProtoType;
use crate::renderer::context::FieldContext;
use crate::renderer::proto::PACKAGE_SEPARATOR;
use crate::renderer::RendererConfig;
use crate::util;

#[derive(Serialize, Deserialize, Clone)]
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
    /// (NOT YET SUPPORTED) Built-in proto option names and types can be seen here:
    /// https://docs.rs/prost-types/latest/prost_types/struct.MessageOptions.html
    ///
    /// Additionally, a few protox-specific options are supported. See the proto files at
    /// `protox/proto_options/protos` for more info.
    #[serde(serialize_with = "serialize_message_options", skip_deserializing)]
    options: Option<MessageOptions>,

    // Config overlays applied to this File.
    // Only available in scripted renderer.
    #[serde(skip)]
    overlays: HashMap<String, serde_yaml::Value>,
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
            overlays: config
                .overlays
                .by_target_opt_clone(&full_name(package, &message.name)),
        };
        Ok(context)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn fields(&self) -> &Vec<FieldContext> {
        &self.fields
    }
    pub fn options(&self) -> &Option<MessageOptions> {
        &self.options
    }
}

impl Overlayed for MessageContext {
    fn overlays(&self) -> &HashMap<String, serde_yaml::Value> {
        &self.overlays
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

fn full_name(package: Option<&String>, name: &Option<String>) -> Option<String> {
    Some(format!("{}.{}", package?, name.as_ref()?))
}

fn name(message: &DescriptorProto, case: Case) -> Result<String> {
    let name = util::str_or_error(&message.name, || "Message has no 'name'".to_owned())?;
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
        fields.push(FieldContext::new(
            field,
            package,
            message.name.as_ref(),
            &map_data,
            config,
        )?);
    }
    Ok(fields)
}

fn collect_map_data(message: &DescriptorProto, package: Option<&String>) -> Result<MapData> {
    let message_name = util::str_or_error(&message.name, || {
        "collect_map_data: No message name.".to_owned()
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
    _options: &Option<MessageOptions>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // let options = match options {
    //     None => return serializer.serialize_none(),
    //     Some(options) => options,
    // };
    let map = HashMap::<String, String>::new();
    // todo builtin options
    debug!("Serializing message options: {:?}", map);
    serializer.collect_map(map)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use prost_types::{DescriptorProto, FieldDescriptorProto};
    use std::collections::HashMap;

    use crate::renderer::case::Case;
    use crate::renderer::context::message::MessageContext;
    use crate::renderer::overlay_config::OverlayConfig;
    use crate::renderer::RendererConfig;

    #[test]
    fn name() -> Result<()> {
        let config = RendererConfig::default();
        let msg_name = "MsgName".to_owned();
        let mut message = DescriptorProto::default();
        message.name = Some(msg_name.clone());
        let context = MessageContext::new(&message, None, &config)?;
        assert_eq!(context.name, msg_name);
        Ok(())
    }

    #[test]
    fn name_with_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.message_name = Case::UpperSnake;
        let msg_name = "msgName".to_owned();
        let mut message = DescriptorProto::default();
        message.name = Some(msg_name.clone());
        let context = MessageContext::new(&message, None, &config)?;
        assert_eq!(context.name, "MSG_NAME");
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let message = DescriptorProto::default();
        let result = MessageContext::new(&message, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn creates_fields_from_proto() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = DescriptorProto::default();
        proto.name = Some("enum_name".to_owned());
        proto.field.push(field("field0"));
        proto.field.push(field("field1"));
        let context = MessageContext::new(&proto, None, &config)?;
        assert_eq!(context.fields.get(0).map(|f| f.name()), Some("field0"));
        assert_eq!(context.fields.get(1).map(|f| f.name()), Some("field1"));
        Ok(())
    }

    #[test]
    fn overlay() -> Result<()> {
        let proto = DescriptorProto {
            name: Some("MessageName".to_owned()),
            ..Default::default()
        };
        let package = "some.package".to_owned();
        let config = RendererConfig {
            overlays: OverlayConfig::new(
                HashMap::new(),
                HashMap::from([(
                    "some.package.MessageName".to_owned(),
                    HashMap::from([(
                        "some_key".to_owned(),
                        serde_yaml::Value::String("some_value".to_owned()),
                    )]),
                )]),
            ),
            ..Default::default()
        };
        let context = MessageContext::new(&proto, Some(&package), &config)?;
        assert_eq!(
            &context.overlays.get("some_key").expect("key did not exist"),
            &"some_value"
        );
        Ok(())
    }

    fn field(name: impl ToString) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.to_string()),
            number: None,
            label: None,
            r#type: None,
            type_name: Some("type".to_owned()),
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }
}
