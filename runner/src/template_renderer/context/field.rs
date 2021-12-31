use anyhow::Result;
use log::debug;
use prost_types::FieldDescriptorProto;
use serde::{Deserialize, Serialize};

use crate::template_renderer::context::message;
use crate::template_renderer::context::proto_type::ProtoType;
use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;

#[derive(Serialize, Deserialize)]
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

    /// This field's type is a map. Use the `*_key_type` and `*_value_type` fields.
    is_map: bool,

    /// When `is_map` is true, equivalent to `fully_qualified_type` for the key type of the map.
    fully_qualified_key_type: Option<String>,

    /// When `is_map` is true, equivalent to `fully_qualified_type` for the value type of the map.
    fully_qualified_value_type: Option<String>,

    /// When `is_map` is true, equivalent to `relative_type` for the key type of the map.
    relative_key_type: Option<String>,

    /// When `is_map` is true, equivalent to `relative_type` for the value type of the map.
    relative_value_type: Option<String>,
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
            fully_qualified_type: Some(type_path.to_string()),
            relative_type: Some(type_path.relative_to(package, parent_prefix)),
            is_map: false,
            fully_qualified_key_type: None,
            fully_qualified_value_type: None,
            relative_key_type: None,
            relative_value_type: None,
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
            is_map: true,
            fully_qualified_key_type: Some(key_type_path.to_string()),
            fully_qualified_value_type: Some(value_type_path.to_string()),
            relative_key_type: Some(key_type_path.relative_to(package, parent_prefix)),
            relative_value_type: Some(value_type_path.relative_to(package, parent_prefix)),
        };
        Ok(context)
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.field_name
    }
}

fn log_new_field(name: &Option<String>) {
    debug!("Creating field context: {}", util::str_or_unknown(name));
}

fn field_name(field: &FieldDescriptorProto, config: &RendererConfig) -> Result<String> {
    let field_name = util::str_or_error(&field.name, || "Field has no 'name'".to_string())?;
    let case = config.case_config.field_name;
    let result = case.rename(
        config
            .field_name_override
            .get(field_name)
            .map(String::as_str)
            .unwrap_or(field_name),
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::case::Case;
    use anyhow::Result;
    use prost_types::FieldDescriptorProto;

    use crate::template_renderer::context::field::FieldContext;
    use crate::template_renderer::context::message;
    use crate::template_renderer::primitive;
    use crate::template_renderer::renderer_config::RendererConfig;

    #[test]
    fn field_name() -> Result<()> {
        let config = RendererConfig::default();
        let name = "test_name".to_string();
        let mut field = default_field();
        field.name = Some(name.clone());
        field.type_name = Some(primitive::FLOAT.to_string());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.field_name.to_string(), name);
        Ok(())
    }

    #[test]
    fn override_field_name() -> Result<()> {
        let old_name = "old_name".to_string();
        let new_name = "new_name".to_string();
        let mut config = RendererConfig::default();
        config
            .field_name_override
            .insert(old_name.clone(), new_name.clone());
        let mut field = default_field();
        field.name = Some(old_name);
        field.type_name = Some(primitive::FLOAT.to_string());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.field_name.to_string(), new_name);
        Ok(())
    }

    #[test]
    fn field_name_case_change() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.field_name = Case::UpperSnake;
        let name = "testName".to_string();
        let mut field = default_field();
        field.name = Some(name.clone());
        field.type_name = Some(primitive::FLOAT.to_string());
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(context.field_name.to_string(), "TEST_NAME");
        Ok(())
    }

    mod type_name_from_config {
        use anyhow::Result;

        use crate::template_renderer::context::field::tests::default_field;
        use crate::template_renderer::context::field::FieldContext;
        use crate::template_renderer::context::message;
        use crate::template_renderer::renderer_config::RendererConfig;

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
                proto_type_name.to_string(),
                ["Test", proto_type_name].concat(),
            );
            let mut field = default_field();
            field.name = Some("field_name".to_string());
            field.type_name = Some(proto_type_name.to_string());
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
        field.name = Some("test".to_string());
        field.type_name = Some(".root.sub.TypeName".to_string());
        let mut config = RendererConfig::default();
        config.package_separator = "::".to_string();
        let context = FieldContext::new(
            &field,
            Some(&"root".to_string()),
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
        field.type_name = Some(primitive::FLOAT.to_string());
        let result = FieldContext::new(&field, None, &message::MapData::new(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn missing_type_name_errors() {
        let config = RendererConfig::default();
        let mut field = default_field();
        field.name = Some("field_name".to_string());
        let result = FieldContext::new(&field, None, &message::MapData::new(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn type_name_case() -> Result<()> {
        let mut config = RendererConfig::default();
        config.case_config.message_name = Case::UpperSnake;
        let mut field = default_field();
        field.name = Some("field_name".to_string());
        field.type_name = Some("TypeName".to_string());
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
        field.name = Some("field_name".to_string());
        field.r#type = Some(2);
        let context = FieldContext::new(&field, None, &message::MapData::new(), &config)?;
        assert_eq!(
            context.fully_qualified_type,
            Some(primitive::FLOAT.to_ascii_lowercase())
        );
        Ok(())
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
