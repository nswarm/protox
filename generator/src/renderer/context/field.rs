use std::collections::HashMap;

use anyhow::Result;
use log::debug;
use prost_types::field_descriptor_proto::Label;
use prost_types::{FieldDescriptorProto, FieldOptions};
use serde::ser::Error;
use serde::{Deserialize, Serialize, Serializer};

use crate::renderer::context::message;
use crate::renderer::context::proto_type::ProtoType;
use crate::renderer::option_key_value::insert_custom_options;
use crate::renderer::RendererConfig;
use crate::util;

#[derive(Serialize, Deserialize, Clone)]
pub struct FieldContext {
    // Name of the field.
    field_name: String,

    /// Type as defined by type config or literal type name. Only valid if `is_map` is false.
    ///
    /// If `is_map` is true, use `*_key_type` and `*_value_type` fields instead.
    ///
    /// ```txt
    ///      pkg.sub_pkg.TypeName
    /// ```
    fully_qualified_type: Option<String>,

    /// Type relative to the owning file's package. Only valid if `is_map` is false.
    ///
    /// If `is_map` is true, use `*_key_type` and `*_value_type` fields instead.
    ///
    /// ```txt
    ///      package:  pkg.sub
    ///      type:     pkg.sub.deep.TypeName
    ///      relative: deep.TypeName
    /// ```
    relative_type: Option<String>,

    /// This field's type is an array of the type specified in `fully_qualified_type` and `relative_type`.
    is_array: bool,

    /// This field's type is a map. Use the `*_key_type` and `*_value_type` fields.
    is_map: bool,

    /// This field is part of a oneof type.
    is_oneof: bool,

    /// When `is_map` is true, equivalent to `fully_qualified_type` for the key type of the map.
    fully_qualified_key_type: Option<String>,

    /// When `is_map` is true, equivalent to `fully_qualified_type` for the value type of the map.
    fully_qualified_value_type: Option<String>,

    /// When `is_map` is true, equivalent to `relative_type` for the key type of the map.
    relative_key_type: Option<String>,

    /// When `is_map` is true, equivalent to `relative_type` for the value type of the map.
    relative_value_type: Option<String>,

    /// Proto field options are serialized as an object like so:
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
    /// https://docs.rs/prost-types/latest/prost_types/struct.FieldOptions.html
    ///
    /// Additionally, a few protox-specific options are supported. See the proto files at
    /// `protox/proto_options/protos` for more info.
    #[serde(serialize_with = "serialize_field_options", skip_deserializing)]
    options: Option<FieldOptions>,
}

impl FieldContext {
    pub fn new(
        field: &FieldDescriptorProto,
        package: Option<&String>,
        map_data: &message::MapData,
        config: &RendererConfig,
    ) -> Result<Self> {
        log_new_field(&field.name);
        match &field.type_name {
            None => FieldContext::new_basic(field, package, config),
            Some(type_name) => match map_data.get(type_name) {
                None => FieldContext::new_basic(field, package, config),
                Some(entry_data) => FieldContext::new_map(field, package, entry_data, config),
            },
        }
    }

    fn new_basic(
        field: &FieldDescriptorProto,
        package: Option<&String>,
        config: &RendererConfig,
    ) -> Result<Self> {
        let type_path = ProtoType::from_field(field)?.to_type_path(config)?;
        let parent_prefix = config.field_relative_parent_prefix.as_ref();
        let context = Self {
            field_name: field_name(field, &config)?,
            fully_qualified_type: Some(type_path.to_owned()),
            relative_type: Some(type_path.relative_to(package, parent_prefix)),
            is_array: is_array(field),
            is_map: false,
            is_oneof: is_oneof(field),
            fully_qualified_key_type: None,
            fully_qualified_value_type: None,
            relative_key_type: None,
            relative_value_type: None,
            options: field.options.clone(),
        };
        Ok(context)
    }

    fn new_map(
        field: &FieldDescriptorProto,
        package: Option<&String>,
        entry: &message::MapEntryData,
        config: &RendererConfig,
    ) -> Result<Self> {
        let key_type_path = entry.key.to_type_path(config)?;
        let value_type_path = entry.value.to_type_path(config)?;
        let parent_prefix = config.field_relative_parent_prefix.as_ref();
        let context = Self {
            field_name: field_name(field, &config)?,
            fully_qualified_type: None,
            relative_type: None,
            is_array: false,
            is_map: true,
            is_oneof: is_oneof(field),
            fully_qualified_key_type: Some(key_type_path.to_owned()),
            fully_qualified_value_type: Some(value_type_path.to_owned()),
            relative_key_type: Some(key_type_path.relative_to(package, parent_prefix)),
            relative_value_type: Some(value_type_path.relative_to(package, parent_prefix)),
            options: field.options.clone(),
        };
        Ok(context)
    }

    pub fn name(&self) -> &str {
        &self.field_name
    }
    pub fn fully_qualified_type(&self) -> Option<&String> {
        self.fully_qualified_type.as_ref()
    }
    pub fn relative_type(&self) -> Option<&String> {
        self.relative_type.as_ref()
    }
    pub fn is_array(&self) -> bool {
        self.is_array
    }
    pub fn is_map(&self) -> bool {
        self.is_map
    }
    pub fn is_oneof(&self) -> bool {
        self.is_oneof
    }
    pub fn fully_qualified_key_type(&self) -> Option<&String> {
        self.fully_qualified_key_type.as_ref()
    }
    pub fn fully_qualified_value_type(&self) -> Option<&String> {
        self.fully_qualified_value_type.as_ref()
    }
    pub fn relative_key_type(&self) -> Option<&String> {
        self.relative_key_type.as_ref()
    }
    pub fn relative_value_type(&self) -> Option<&String> {
        self.relative_value_type.as_ref()
    }
    pub fn options(&self) -> Option<&FieldOptions> {
        self.options.as_ref()
    }
}

fn log_new_field(name: &Option<String>) {
    debug!("Creating field context: {}", util::str_or_unknown(name));
}

fn field_name(field: &FieldDescriptorProto, config: &RendererConfig) -> Result<String> {
    let field_name = util::str_or_error(&field.name, || "Field has no 'name'".to_owned())?;
    let case = config.case_config.field_name;
    let renamed = case.rename(field_name);
    let result = config
        .field_name_override
        .get(&renamed)
        .map(String::clone)
        .unwrap_or(renamed);
    Ok(result)
}

fn is_array(field: &FieldDescriptorProto) -> bool {
    field
        .label
        .map(|label| label == Label::Repeated as i32)
        .unwrap_or(false)
}

fn is_oneof(field: &FieldDescriptorProto) -> bool {
    field.oneof_index.is_some()
}

fn serialize_field_options<S: Serializer>(
    options: &Option<FieldOptions>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let options = match options {
        None => return serializer.serialize_none(),
        Some(options) => options,
    };
    let mut map = HashMap::new();
    insert_custom_options(&mut map, options, &proto_options::FIELD_KEY_VALUE)
        .map_err(|err| S::Error::custom(err.to_string()))?;
    debug!("Serializing field options: {:?}", map);
    serializer.collect_map(map)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::field_descriptor_proto::Label;
    use prost_types::{FieldDescriptorProto, FieldOptions};

    use crate::renderer::case::Case;
    use crate::renderer::context::field::FieldContext;
    use crate::renderer::context::message;
    use crate::renderer::context::message::MapData;
    use crate::renderer::primitive;
    use crate::renderer::RendererConfig;

    #[test]
    fn field_name() -> Result<()> {
        let config = RendererConfig::default();
        let name = "test_name".to_owned();
        let mut field = default_field();
        field.name = Some(name.clone());
        field.type_name = Some(primitive::FLOAT.to_owned());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.field_name.to_owned(), name);
        Ok(())
    }

    #[test]
    fn override_field_name() -> Result<()> {
        let old_name = "bad_name".to_owned();
        let new_name = "good_name".to_owned();
        let mut config = RendererConfig::default();
        config.case_config.field_name = Case::LowerSnake;
        // Note override takes place AFTER case adjustment
        config
            .field_name_override
            .insert(old_name.clone(), new_name.clone());
        let mut field = default_field();
        field.name = Some(old_name);
        field.type_name = Some(primitive::FLOAT.to_owned());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.field_name.to_owned(), new_name);
        Ok(())
    }

    #[test]
    fn field_name_case_change() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.field_name = Case::UpperSnake;
        let name = "testName".to_owned();
        let mut field = default_field();
        field.name = Some(name.clone());
        field.type_name = Some(primitive::FLOAT.to_owned());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.field_name.to_owned(), "TEST_NAME");
        Ok(())
    }

    #[test]
    fn key_value_options() -> Result<()> {
        let config = RendererConfig::default();
        let mut field = default_field();
        field.name = Some("field_name".to_owned());
        field.type_name = Some(primitive::FLOAT.to_owned());
        let mut options = FieldOptions::default();
        options.set_extension_data(
            &proto_options::FIELD_KEY_VALUE,
            vec!["key0=value0".to_owned(), "key1=value1".to_owned()],
        )?;
        field.options = Some(options);

        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        let json = serde_json::to_string(&context)?;
        println!("{}", json);
        assert!(json.contains(r#""key0":"value0""#));
        assert!(json.contains(r#""key1":"value1""#));
        Ok(())
    }

    #[test]
    fn native_type_option() -> Result<()> {
        let expected_type = "custom_type";
        let config = RendererConfig::default();
        let mut field = default_field();
        field.name = Some("field_name".to_owned());
        field.type_name = Some(primitive::FLOAT.to_owned());
        let mut options = FieldOptions::default();
        options.set_extension_data(&proto_options::NATIVE_TYPE, expected_type.to_owned())?;
        field.options = Some(options);

        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.relative_type, Some("custom_type".to_owned()));
        Ok(())
    }

    mod type_name_from_config {
        use anyhow::Result;

        use crate::renderer::context::field::tests::default_field;
        use crate::renderer::context::field::FieldContext;
        use crate::renderer::context::message;
        use crate::renderer::RendererConfig;

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
                proto_type_name.to_owned(),
                ["Test", proto_type_name].concat(),
            );
            let mut field = default_field();
            field.name = Some("field_name".to_owned());
            field.type_name = Some(proto_type_name.to_owned());
            let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
            assert_eq!(
                context.fully_qualified_type.as_ref(),
                config.type_config.get(proto_type_name),
            );
            Ok(())
        }
    }

    #[test]
    fn package_separator_replaced_in_types() -> Result<()> {
        let mut field = default_field();
        field.name = Some("test".to_owned());
        field.type_name = Some(".root.sub.TypeName".to_owned());
        let mut config = RendererConfig::default();
        config.package_separator = "::".to_owned();
        let context = FieldContext::new(
            &field,
            Some(&"root".to_owned()),
            &message::MapData::new(),
            &config,
        )?;
        assert_eq!(
            context.relative_type.as_ref().map(String::as_str),
            Some("sub::TypeName")
        );
        assert_eq!(
            context.fully_qualified_type.as_ref().map(String::as_str),
            Some("root::sub::TypeName")
        );
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let mut field = default_field();
        field.type_name = Some(primitive::FLOAT.to_owned());
        let result = FieldContext::new(&field, None, &message::MapData::new(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn missing_type_name_errors() {
        let config = RendererConfig::default();
        let mut field = default_field();
        field.name = Some("field_name".to_owned());
        let result = FieldContext::new(&field, None, &message::MapData::new(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn type_name_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.message_name = Case::UpperSnake;
        let mut field = default_field();
        field.name = Some("field_name".to_owned());
        field.type_name = Some("TypeName".to_owned());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(
            context.fully_qualified_type.as_ref().map(String::as_str),
            Some("TYPE_NAME")
        );
        Ok(())
    }

    #[test]
    fn type_name_case_ignored_for_primitives() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.message_name = Case::UpperSnake;
        let mut field = default_field();
        field.name = Some("field_name".to_owned());
        field.r#type = Some(2);
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(
            context.fully_qualified_type,
            Some(primitive::FLOAT.to_ascii_lowercase())
        );
        Ok(())
    }

    #[test]
    fn array() -> Result<()> {
        let mut field = field_with_required();
        field.label = Some(Label::Repeated as i32);
        let config = RendererConfig::default();
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert!(context.is_array);
        Ok(())
    }

    mod map {
        use anyhow::Result;
        use prost_types::FieldDescriptorProto;

        use crate::renderer::context::field::tests::field_with_required;
        use crate::renderer::context::message::MapEntryData;
        use crate::renderer::context::proto_type::{primitive_type_name, ProtoType};
        use crate::renderer::context::{message, FieldContext};
        use crate::renderer::RendererConfig;

        #[test]
        fn complex_value() -> Result<()> {
            let field = map_field();
            let config = RendererConfig::default();
            let mut map_data = message::MapData::new();
            let int_proto_type = prost_types::field::Kind::TypeInt32 as i32;
            let package = ".root.sub".to_owned();
            map_data.insert(
                MAP_TYPE_NAME.to_owned(),
                MapEntryData {
                    key: ProtoType::Type(int_proto_type),
                    value: ProtoType::TypeName(".root.sub.inner.TypeName".to_owned()),
                },
            );

            let expected_key = primitive_type_name(int_proto_type, &config)?;
            let context = FieldContext::new(&field, Some(&package), &map_data, &config)?;
            assert!(context.is_map);
            assert_eq!(
                context.fully_qualified_key_type,
                Some(expected_key.to_owned())
            );
            assert_eq!(
                context.fully_qualified_value_type,
                Some("root.sub.inner.TypeName".to_owned())
            );
            assert_eq!(context.relative_key_type, Some(expected_key.to_owned()));
            assert_eq!(
                context.relative_value_type,
                Some("inner.TypeName".to_owned())
            );
            Ok(())
        }

        #[test]
        fn primitive_key_value() -> Result<()> {
            let field = map_field();
            let config = RendererConfig::default();
            let mut map_data = message::MapData::new();
            let int_proto_type = prost_types::field::Kind::TypeInt32 as i32;
            let float_proto_type = prost_types::field::Kind::TypeFloat as i32;
            map_data.insert(
                MAP_TYPE_NAME.to_owned(),
                MapEntryData {
                    key: ProtoType::Type(int_proto_type),
                    value: ProtoType::Type(float_proto_type),
                },
            );

            let expected_key = primitive_type_name(int_proto_type, &config)?;
            let expected_value = primitive_type_name(float_proto_type, &config)?;
            let context = FieldContext::new(&field, None, &map_data, &config)?;
            assert!(context.is_map);
            assert_eq!(
                context.fully_qualified_key_type,
                Some(expected_key.to_owned())
            );
            assert_eq!(
                context.fully_qualified_value_type,
                Some(expected_value.to_owned())
            );
            assert_eq!(context.relative_key_type, Some(expected_key.to_owned()));
            assert_eq!(context.relative_value_type, Some(expected_value.to_owned()));
            Ok(())
        }

        #[test]
        fn non_map_has_no_map_fields() -> Result<()> {
            let field = field_with_required();
            let config = RendererConfig::default();
            let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
            assert!(!context.is_map);
            assert!(context.fully_qualified_key_type.is_none());
            assert!(context.fully_qualified_value_type.is_none());
            assert!(context.relative_key_type.is_none());
            assert!(context.relative_value_type.is_none());
            Ok(())
        }

        const MAP_TYPE_NAME: &str = ".MapType";

        fn map_field() -> FieldDescriptorProto {
            let mut field = field_with_required();
            field.type_name = Some(MAP_TYPE_NAME.to_owned());
            field
        }
    }

    #[test]
    fn is_oneof_field() -> Result<()> {
        let config = RendererConfig::default();
        let mut field = field_with_required();
        field.oneof_index = Some(0);
        let context = FieldContext::new(&field, None, &MapData::new(), &config)?;
        assert!(context.is_oneof);
        Ok(())
    }

    fn field_with_required() -> FieldDescriptorProto {
        let mut field = default_field();
        field.name = Some("field_name".to_owned());
        field.r#type = Some(2);
        field
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
