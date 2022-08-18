use std::collections::HashMap;

use crate::renderer::context::overlayed::Overlayed;
use crate::renderer::option_key_value::insert_custom_options;
use anyhow::{anyhow, Result};
use log::debug;
use prost_types::{EnumDescriptorProto, EnumOptions, EnumValueDescriptorProto, EnumValueOptions};
use serde::ser::Error;
use serde::{Deserialize, Serialize, Serializer};

use crate::renderer::RendererConfig;
use crate::util;

#[derive(Serialize, Deserialize, Clone)]
pub struct EnumContext {
    // Name of this enum.
    name: String,

    // Values defined by this enum.
    values: Vec<EnumValueContext>,

    /// Proto enum options are serialized as an object like so:
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
    /// https://docs.rs/prost-types/latest/prost_types/struct.EnumOptions.html
    ///
    /// Additionally, a few protox-specific options are supported. See the proto files at
    /// `protox/proto_options/protos` for more info.
    #[serde(serialize_with = "serialize_enum_options", skip_deserializing)]
    options: Option<EnumOptions>,

    // Config overlays applied to this File.
    // Only available in scripted renderer.
    #[serde(skip)]
    overlays: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EnumValueContext {
    name: String,
    number: i32,

    /// Currently only supported in scripted renderer.
    #[serde(skip)]
    options: Option<EnumValueOptions>,

    // Config overlays applied to this File.
    // Only available in scripted renderer.
    #[serde(skip)]
    overlays: HashMap<String, serde_yaml::Value>,
}

impl EnumContext {
    pub fn new(
        proto: &EnumDescriptorProto,
        package: Option<&String>,
        config: &RendererConfig,
    ) -> Result<Self> {
        log_new_enum(&proto.name);
        let context = Self {
            name: name(&proto, config)?,
            values: values(&proto, package, config)?,
            options: proto.options.clone(),
            overlays: config
                .overlays
                .by_target_opt_clone(&full_name(package, &proto.name)),
        };
        Ok(context)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn values(&self) -> &[EnumValueContext] {
        &self.values
    }
    pub fn options(&self) -> &Option<EnumOptions> {
        &self.options
    }
}

impl EnumValueContext {
    pub fn new(
        proto: &EnumValueDescriptorProto,
        message_full_name: Option<&String>,
        config: &RendererConfig,
    ) -> Result<Self> {
        let (name, number) = match (proto.name.clone(), proto.number) {
            (Some(name), Some(number)) => (name, number),
            _ => return Err(error_invalid_value(&proto.name)),
        };
        let case = &config.case_config.enum_value_name;
        Ok(EnumValueContext {
            name: case.rename(&name),
            number,
            options: proto.options.clone(),
            overlays: config
                .overlays
                .by_target_opt_clone(&full_name(message_full_name, &proto.name)),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn number(&self) -> i32 {
        self.number
    }
    pub fn options(&self) -> &Option<EnumValueOptions> {
        &self.options
    }
}

impl Overlayed for EnumContext {
    fn overlays(&self) -> &HashMap<String, serde_yaml::Value> {
        &self.overlays
    }
}

impl Overlayed for EnumValueContext {
    fn overlays(&self) -> &HashMap<String, serde_yaml::Value> {
        &self.overlays
    }
}

fn log_new_enum(name: &Option<String>) {
    debug!("Creating message context: {}", util::str_or_unknown(name));
}

fn full_name(package: Option<&String>, name: &Option<String>) -> Option<String> {
    Some(format!("{}.{}", package?, name.as_ref()?))
}

fn name(proto: &EnumDescriptorProto, config: &RendererConfig) -> Result<String> {
    let name = util::str_or_error(&proto.name, || "Enum has no 'name'".to_owned())?;
    Ok(config.case_config.enum_name.rename(name))
}

fn values(
    proto: &EnumDescriptorProto,
    package: Option<&String>,
    config: &RendererConfig,
) -> Result<Vec<EnumValueContext>> {
    let mut values = Vec::new();
    for proto_value in &proto.value {
        let message_full_name = full_name(package, &proto.name);
        values.push(EnumValueContext::new(
            proto_value,
            message_full_name.as_ref(),
            config,
        )?);
    }
    Ok(values)
}

fn error_invalid_value(name: &Option<String>) -> anyhow::Error {
    anyhow!(
        "Enum '{}' has a value missing name and/or number.",
        util::str_or_unknown(name),
    )
}

fn serialize_enum_options<S: Serializer>(
    options: &Option<EnumOptions>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let options = match options {
        None => return serializer.serialize_none(),
        Some(options) => options,
    };
    let mut map = HashMap::new();
    insert_custom_options(&mut map, options, &proto_options::ENUM_KEY_VALUE)
        .map_err(|err| S::Error::custom(err.to_string()))?;
    debug!("Serializing enum options: {:?}", map);
    serializer.collect_map(map)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::{EnumDescriptorProto, EnumOptions, EnumValueDescriptorProto};
    use std::collections::HashMap;

    use crate::renderer::case::Case;
    use crate::renderer::context::{EnumContext, EnumValueContext};
    use crate::renderer::overlay_config::OverlayConfig;
    use crate::renderer::RendererConfig;

    #[test]
    fn name() -> Result<()> {
        let config = RendererConfig::default();
        let enum_name = "MsgName".to_owned();
        let mut proto = EnumDescriptorProto::default();
        proto.name = Some(enum_name.clone());
        let context = EnumContext::new(&proto, None, &config)?;
        assert_eq!(context.name, enum_name);
        Ok(())
    }

    #[test]
    fn name_with_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.enum_name = Case::UpperSnake;
        let enum_name = "MsgName".to_owned();
        let mut proto = EnumDescriptorProto::default();
        proto.name = Some(enum_name.clone());
        let context = EnumContext::new(&proto, None, &config)?;
        assert_eq!(context.name, "MSG_NAME");
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let proto = EnumDescriptorProto::default();
        let result = EnumContext::new(&proto, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn values() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = EnumDescriptorProto::default();
        proto.name = Some("EnumName".to_owned());
        proto.value.push(enum_value(1));
        proto.value.push(enum_value(2));
        let context = EnumContext::new(&proto, None, &config)?;
        assert_eq!(context.values[0].name, "1");
        assert_eq!(context.values[0].number, 1);
        assert_eq!(context.values[1].name, "2");
        assert_eq!(context.values[1].number, 2);
        Ok(())
    }

    #[test]
    fn values_with_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.enum_value_name = Case::UpperSnake;
        let mut proto = EnumDescriptorProto::default();
        proto.name = Some("EnumName".to_owned());
        proto.value.push(named_enum_value("ValueName1", 1));
        proto.value.push(named_enum_value("ValueName2", 2));
        let context = EnumContext::new(&proto, None, &config)?;
        assert_eq!(context.values[0].name, "VALUE_NAME1");
        assert_eq!(context.values[0].number, 1);
        assert_eq!(context.values[1].name, "VALUE_NAME2");
        assert_eq!(context.values[1].number, 2);
        Ok(())
    }

    #[test]
    fn key_value_options() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = EnumDescriptorProto::default();
        proto.name = Some("EnumName".to_owned());
        let mut options = EnumOptions::default();
        options.set_extension_data(
            &proto_options::ENUM_KEY_VALUE,
            vec!["key0=value0".to_owned(), "key1=value1".to_owned()],
        )?;
        proto.options = Some(options);

        let context = EnumContext::new(&proto, None, &config)?;
        let json = serde_json::to_string(&context)?;
        println!("{}", json);
        assert!(json.contains(r#""key0":"value0""#));
        assert!(json.contains(r#""key1":"value1""#));
        Ok(())
    }

    #[test]
    fn overlay_enum() -> Result<()> {
        let proto = EnumDescriptorProto {
            name: Some("EnumName".to_owned()),
            ..Default::default()
        };
        let package = "some.package".to_owned();
        let config = RendererConfig {
            overlays: OverlayConfig::new(
                HashMap::new(),
                HashMap::from([(
                    "some.package.EnumName".to_owned(),
                    HashMap::from([(
                        "some_key".to_owned(),
                        serde_yaml::Value::String("some_value".to_owned()),
                    )]),
                )]),
            ),
            ..Default::default()
        };
        let context = EnumContext::new(&proto, Some(&package), &config)?;
        assert_eq!(
            &context.overlays.get("some_key").expect("key did not exist"),
            &"some_value"
        );
        Ok(())
    }

    #[test]
    fn overlay_enum_value() -> Result<()> {
        let proto = named_enum_value("ValueName", 1);
        let message_name = "some.package.EnumName".to_owned();
        let config = RendererConfig {
            overlays: OverlayConfig::new(
                HashMap::new(),
                HashMap::from([(
                    "some.package.EnumName.ValueName".to_owned(),
                    HashMap::from([(
                        "some_key".to_owned(),
                        serde_yaml::Value::String("some_value".to_owned()),
                    )]),
                )]),
            ),
            ..Default::default()
        };
        let context = EnumValueContext::new(&proto, Some(&message_name), &config)?;
        assert_eq!(
            &context.overlays.get("some_key").expect("key did not exist"),
            &"some_value"
        );
        Ok(())
    }

    fn enum_value(number: i32) -> EnumValueDescriptorProto {
        EnumValueDescriptorProto {
            name: Some(number.to_string()),
            number: Some(number),
            options: None,
        }
    }

    fn named_enum_value(name: &str, number: i32) -> EnumValueDescriptorProto {
        EnumValueDescriptorProto {
            name: Some(name.to_owned()),
            number: Some(number),
            options: None,
        }
    }
}
