use crate::generator::context::{FieldContext, FileContext, MessageContext};
use crate::generator::template_config::TemplateConfig;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};
use serde::Serialize;
use std::io;

pub struct Renderer<'a> {
    hbs: Handlebars<'a>,
    config: TemplateConfig,
}

const FILE_TEMPLATE_NAME: &str = "file";
const MESSAGE_TEMPLATE_NAME: &str = "message";
const FIELD_TEMPLATE_NAME: &str = "field";

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

    pub fn with_config(config: TemplateConfig) -> Self {
        Self {
            hbs: Handlebars::new(),
            config,
        }
    }

    pub fn configure(&mut self, config: TemplateConfig) {
        self.config = config;
    }

    pub fn load_file_template(&mut self, template: String) -> Result<()> {
        self.load_template(FILE_TEMPLATE_NAME, template)
    }

    pub fn load_message_template(&mut self, template: String) -> Result<()> {
        self.load_template(MESSAGE_TEMPLATE_NAME, template)
    }

    pub fn load_field_template(&mut self, template: String) -> Result<()> {
        self.load_template(FIELD_TEMPLATE_NAME, template)
    }

    fn load_template(&mut self, name: &str, template: String) -> Result<()> {
        self.hbs
            .register_template_string(name, template)
            .context("Failed to load field template")?;
        Ok(())
    }

    pub fn render_file<W: io::Write>(
        &self,
        file: &FileDescriptorProto,
        writer: &mut W,
    ) -> Result<()> {
        let mut context = FileContext::new(file, &self.config)?;
        for message in &file.message_type {
            context.messages.push(self.render_message(message)?);
        }
        self.render_to_write(FILE_TEMPLATE_NAME, &context, writer)
    }

    fn render_message(&self, message: &DescriptorProto) -> Result<String> {
        let mut context = MessageContext::new(message, &self.config)?;
        for field in &message.field {
            context.fields.push(self.render_field(field)?);
        }
        self.render_to_string(MESSAGE_TEMPLATE_NAME, &context)
    }

    fn render_field(&self, field: &FieldDescriptorProto) -> Result<String> {
        let context = FieldContext::new(field, &self.config)?;
        self.render_to_string(FIELD_TEMPLATE_NAME, &context)
    }

    fn render_to_string<S: Serialize>(&self, template: &str, data: &S) -> Result<String> {
        let rendered = self
            .hbs
            .render(template, data)
            .with_context(|| render_error_context(template, data))?;
        Ok(rendered)
    }

    fn render_to_write<S: Serialize, W: io::Write>(
        &self,
        template: &str,
        data: &S,
        writer: W,
    ) -> Result<()> {
        self.hbs
            .render_to_write(template, data, writer)
            .with_context(|| render_error_context(template, data))?;
        Ok(())
    }
}

fn render_error_context<S: Serialize>(name: &str, data: &S) -> String {
    format!(
        "Failed to render template '{}' for data: {}",
        name,
        serde_json::to_string(data).unwrap_or("(failed to serialize)".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use crate::generator::primitive;
    use crate::generator::renderer::Renderer;
    use crate::generator::template_config::TemplateConfig;
    use anyhow::Result;
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto};

    #[test]
    fn file_template() -> Result<()> {
        let config = TemplateConfig::default();
        let mut renderer = Renderer::with_config(config);
        renderer.load_file_template("{{name}}{{#each messages}}{{this}}{{/each}}".to_string());
        renderer.load_message_template("{{name}}".to_string());
        renderer.load_field_template("{{name}}".to_string());

        let file_name = "file_name".to_string();
        let msg0 = fake_message("msg0", Vec::new());
        let msg1 = fake_message("msg1", Vec::new());
        let msg0_rendered = renderer.render_message(&msg0)?;
        let msg1_rendered = renderer.render_message(&msg1)?;
        let file = fake_file(&file_name, vec![msg0, msg1]);

        let mut bytes = Vec::<u8>::new();
        renderer.render_file(&file, &mut bytes)?;

        let result = String::from_utf8(bytes)?;
        assert_eq!(result, [file_name, msg0_rendered, msg1_rendered].concat());
        Ok(())
    }

    #[test]
    fn message_template() -> Result<()> {
        let config = TemplateConfig::default();
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
        let mut config = TemplateConfig::default();
        config
            .type_config
            .insert(primitive::FLOAT.to_string(), native_type.clone());
        let mut renderer = Renderer::with_config(config);
        renderer.load_field_template(["{{name}}", separator, "{{type_name}}"].concat())?;
        let result = renderer.render_field(&fake_field("field-name", primitive::FLOAT))?;
        assert_eq!(result, [field_name, separator, &native_type].concat());
        Ok(())
    }

    fn fake_file(name: impl Into<String>, messages: Vec<DescriptorProto>) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.into()),
            package: None,
            dependency: vec![],
            public_dependency: vec![],
            weak_dependency: vec![],
            message_type: messages,
            enum_type: vec![],
            service: vec![],
            extension: vec![],
            options: None,
            source_code_info: None,
            syntax: None,
        }
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
