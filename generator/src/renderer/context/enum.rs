use std::collections::HashMap;

use crate::renderer::option_key_value::insert_custom_options;
use anyhow::{anyhow, Result};
use log::debug;
use prost_types::{EnumDescriptorProto, EnumOptions, EnumValueOptions};
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EnumValueContext {
    name: String,
    number: i32,

    /// Currently only supported in scripted renderer.
    #[serde(skip)]
    options: Option<EnumValueOptions>,
}

impl EnumContext {
    pub fn new(proto: &EnumDescriptorProto, config: &RendererConfig) -> Result<Self> {
        log_new_enum(&proto.name);
        let context = Self {
            name: name(&proto, config)?,
            values: values(&proto, config)?,
            options: proto.options.clone(),
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

fn log_new_enum(name: &Option<String>) {
    debug!("Creating message context: {}", util::str_or_unknown(name));
}

fn name(proto: &EnumDescriptorProto, config: &RendererConfig) -> Result<String> {
    let name = util::str_or_error(&proto.name, || "Enum has no 'name'".to_owned())?;
    Ok(config.case_config.enum_name.rename(name))
}

fn values(proto: &EnumDescriptorProto, config: &RendererConfig) -> Result<Vec<EnumValueContext>> {
    let mut values = Vec::new();
    for value in &proto.value {
        let (name, number) = match (value.name.clone(), value.number) {
            (Some(name), Some(number)) => (name, number),
            _ => return Err(error_invalid_value(&value.name)),
        };
        let case = &config.case_config.enum_value_name;
        values.push(EnumValueContext {
            name: case.rename(&name),
            number,
            options: value.options.clone(),
        });
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

    use crate::renderer::case::Case;
    use crate::renderer::context::EnumContext;
    use crate::renderer::RendererConfig;

    #[test]
    fn name() -> Result<()> {
        let config = RendererConfig::default();
        let enum_name = "MsgName".to_owned();
        let mut proto = default_enum();
        proto.name = Some(enum_name.clone());
        let context = EnumContext::new(&proto, &config)?;
        assert_eq!(context.name, enum_name);
        Ok(())
    }

    #[test]
    fn name_with_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.enum_name = Case::UpperSnake;
        let enum_name = "MsgName".to_owned();
        let mut proto = default_enum();
        proto.name = Some(enum_name.clone());
        let context = EnumContext::new(&proto, &config)?;
        assert_eq!(context.name, "MSG_NAME");
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
    fn values() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = default_enum();
        proto.name = Some("EnumName".to_owned());
        proto.value.push(enum_value(1));
        proto.value.push(enum_value(2));
        let context = EnumContext::new(&proto, &config)?;
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
        let mut proto = default_enum();
        proto.name = Some("EnumName".to_owned());
        proto.value.push(named_enum_value("ValueName1", 1));
        proto.value.push(named_enum_value("ValueName2", 2));
        let context = EnumContext::new(&proto, &config)?;
        assert_eq!(context.values[0].name, "VALUE_NAME1");
        assert_eq!(context.values[0].number, 1);
        assert_eq!(context.values[1].name, "VALUE_NAME2");
        assert_eq!(context.values[1].number, 2);
        Ok(())
    }

    #[test]
    fn key_value_options() -> Result<()> {
        let config = RendererConfig::default();
        let mut proto = default_enum();
        proto.name = Some("EnumName".to_owned());
        let mut options = EnumOptions::default();
        options.set_extension_data(
            &proto_options::ENUM_KEY_VALUE,
            vec!["key0=value0".to_owned(), "key1=value1".to_owned()],
        )?;
        proto.options = Some(options);

        let context = EnumContext::new(&proto, &config)?;
        let json = serde_json::to_string(&context)?;
        println!("{}", json);
        assert!(json.contains(r#""key0":"value0""#));
        assert!(json.contains(r#""key1":"value1""#));
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
