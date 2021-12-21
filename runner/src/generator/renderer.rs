use crate::generator::config::Config;
use crate::generator::context::{FieldContext, MessageContext};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use prost_types::{DescriptorProto, FieldDescriptorProto};
use serde::Serialize;

pub struct Renderer<'a> {
    hbs: Handlebars<'a>,
    config: Config,
}

const FIELD_TEMPLATE_NAME: &str = "field";
const MESSAGE_TEMPLATE_NAME: &str = "message";

/// Renders final output files by using data from a proto descriptor set,
/// a set of templates which define how the data is structured, and the generator
/// configuration, which supplies user choices about how to handle specifics.
impl Renderer<'_> {
    pub fn new() -> Self {
        Self {
            hbs: Handlebars::new(),
            config: Default::default(),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            hbs: Handlebars::new(),
            config,
        }
    }

    pub fn configure(&mut self, config: Config) {
        self.config = config;
    }

    pub fn load_field_template(&mut self, template: String) -> Result<()> {
        self.load_template(FIELD_TEMPLATE_NAME, template)
    }

    pub fn load_message_template(&mut self, template: String) -> Result<()> {
        self.load_template(MESSAGE_TEMPLATE_NAME, template)
    }

    fn load_template(&mut self, name: &str, template: String) -> Result<()> {
        self.hbs
            .register_template_string(name, template)
            .context("Failed to load field template")?;
        Ok(())
    }

    fn render_message(&self, message: &DescriptorProto) -> Result<String> {
        let mut context = MessageContext::new(message, &self.config)?;
        for field in &message.field {
            context.fields.push(self.render_field(field)?);
        }
        self.render(MESSAGE_TEMPLATE_NAME, &context)
    }

    fn render_field(&self, field: &FieldDescriptorProto) -> Result<String> {
        let context = FieldContext::new(field, &self.config)?;
        self.render(FIELD_TEMPLATE_NAME, &context)
    }

    fn render<S: Serialize>(&self, template: &str, data: &S) -> Result<String> {
        let rendered = self.hbs.render(template, data).with_context(|| {
            format!(
                "Failed to render template for data: {}",
                serde_json::to_string(data).unwrap_or("(failed to serialize)".to_string()),
            )
        })?;
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::config::Config;
    use crate::generator::primitive;
    use crate::generator::renderer::Renderer;
    use anyhow::Result;
    use prost_types::{DescriptorProto, FieldDescriptorProto};

    #[test]
    fn message_template() -> Result<()> {
        let config = Config::default();
        let mut renderer = Renderer::with_config(config);
        renderer.load_message_template("{{name}}{{#each fields}}{{this}}{{/each}}".to_string());
        renderer.load_field_template("{{name}}:::{{type_name}}".to_string());

        let msg_name = "msg_name".to_string();
        let field0 = fake_field("field0", primitive::FLOAT);
        let field1 = fake_field("field1", primitive::BOOL);
        let field0_rendered = renderer.render_field(&field0)?;
        let field1_rendered = renderer.render_field(&field1)?;
        let message = fake_message(&msg_name, vec![field0, field1]);

        let result = renderer.render_message(&message)?;
        assert_eq!(
            result,
            [msg_name, field0_rendered, field1_rendered].concat()
        );
        Ok(())
    }

    #[test]
    fn field_template() -> Result<()> {
        let field_name = "field-name";
        let native_type = ["TEST-", primitive::FLOAT].concat();
        let separator = ":::";
        let mut config = Config::default();
        config
            .type_config
            .insert(primitive::FLOAT.to_string(), native_type.clone());
        let mut renderer = Renderer::with_config(config);
        renderer.load_field_template(["{{name}}", separator, "{{type_name}}"].concat())?;
        let result = renderer.render_field(&fake_field("field-name", primitive::FLOAT))?;
        assert_eq!(result, [field_name, separator, &native_type].concat());
        Ok(())
    }

    fn fake_message(name: impl Into<String>, fields: Vec<FieldDescriptorProto>) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.into()),
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

    fn fake_field(name: impl Into<String>, type_name: impl Into<String>) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.into()),
            number: None,
            label: None,
            r#type: None,
            type_name: Some(type_name.into()),
            extendee: None,
            default_value: None,
            oneof_index: None,
            json_name: None,
            options: None,
            proto3_optional: None,
        }
    }
}
