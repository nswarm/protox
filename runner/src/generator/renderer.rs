use crate::generator::context::{FieldContext, FileContext, MessageContext};
use crate::generator::template_config::TemplateConfig;
use crate::{util, DisplayNormalized};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use log::debug;
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::{fs, io};

const FILE_TEMPLATE_NAME: &str = "file";
const MESSAGE_TEMPLATE_NAME: &str = "message";
const FIELD_TEMPLATE_NAME: &str = "field";
const CONFIG_FILE_NAME: &str = "config.json";

/// Renders final output files by using:
/// 1. Data from a proto descriptor set.
/// 2. A set of templates which define how the data is structured.
/// 3. A template configuration file, which supplies user choices about how to handle specifics.
pub struct Renderer<'a> {
    hbs: Handlebars<'a>,
    config: TemplateConfig,
}

impl Renderer<'_> {
    pub fn new() -> Self {
        Self {
            hbs: Handlebars::new(),
            config: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: TemplateConfig) -> Self {
        Self {
            hbs: Handlebars::new(),
            config,
        }
    }

    /// Loads config and templates from the same root path with the following names:
    /// ```txt
    ///     root/config.json
    ///     root/file.hbs
    ///     root/message.hbs
    ///     root/field.hbs
    /// ```
    pub fn load_all(&mut self, root: &Path) -> Result<()> {
        self.load_config(&root.join(CONFIG_FILE_NAME))?;
        self.load_templates(root)?;
        Ok(())
    }

    pub fn load_config(&mut self, path: &Path) -> Result<TemplateConfig> {
        let file = fs::File::open(path).context("Failed to read template config file.")?;
        let buf_reader = io::BufReader::new(file);
        let config = serde_json::from_reader(buf_reader).with_context(|| {
            format!(
                "Failed to deserialize template config as json, path: {}",
                path.display_normalized()
            )
        })?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn load_file_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(FILE_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    pub fn load_message_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(MESSAGE_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    pub fn load_field_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(FIELD_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    fn load_template_string(&mut self, name: &str, template: String) -> Result<()> {
        self.hbs
            .register_template_string(name, template)
            .with_context(|| format!("Failed to load '{}' template from string", name))?;
        Ok(())
    }

    pub fn load_templates(&mut self, root: &Path) -> Result<()> {
        self.load_file_template_file(&hbs_file_path(root, FILE_TEMPLATE_NAME))?;
        self.load_message_template_file(&hbs_file_path(root, MESSAGE_TEMPLATE_NAME))?;
        self.load_field_template_file(&hbs_file_path(root, FIELD_TEMPLATE_NAME))?;
        Ok(())
    }

    pub fn load_file_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(FILE_TEMPLATE_NAME, path)
    }

    pub fn load_message_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(MESSAGE_TEMPLATE_NAME, path)
    }

    pub fn load_field_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(FIELD_TEMPLATE_NAME, path)
    }

    fn load_template_file(&mut self, name: &str, path: &Path) -> Result<()> {
        self.hbs
            .register_template_file(name, path)
            .with_context(|| {
                format!(
                    "Failed to load '{}' template at path: {}",
                    name,
                    path.display_normalized()
                )
            })?;
        Ok(())
    }

    pub fn render_file<W: io::Write>(
        &self,
        file: &FileDescriptorProto,
        writer: &mut W,
    ) -> Result<()> {
        debug!("Rendering file: {}", util::str_or_unknown(&file.name));
        let mut context = FileContext::new(file, &self.config)?;
        for message in &file.message_type {
            context.messages.push(self.render_message(message)?);
        }
        self.render_to_write(FILE_TEMPLATE_NAME, &context, writer)
    }

    fn render_message(&self, message: &DescriptorProto) -> Result<String> {
        debug!("Rendering message: {}", util::str_or_unknown(&message.name));
        let mut context = MessageContext::new(message, &self.config)?;
        for field in &message.field {
            context.fields.push(self.render_field(field)?);
        }
        self.render_to_string(MESSAGE_TEMPLATE_NAME, &context)
    }

    fn render_field(&self, field: &FieldDescriptorProto) -> Result<String> {
        debug!("Rendering field: {}", util::str_or_unknown(&field.name));
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

fn hbs_file_path(root: &Path, file_name: &str) -> PathBuf {
    static HBS_EXT: &str = "hbs";
    let mut path = root.join(file_name);
    path.set_extension(HBS_EXT);
    path
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
        renderer
            .load_file_template_string("{{name}}{{#each messages}}{{this}}{{/each}}".to_string())?;
        renderer.load_message_template_string("{{name}}".to_string())?;
        renderer.load_field_template_string("{{name}}".to_string())?;

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
        renderer.load_message_template_string(
            "{{name}}{{#each fields}}{{this}}{{/each}}".to_string(),
        )?;
        renderer.load_field_template_string("{{name}}:::{{native_type}}".to_string())?;

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
        renderer.load_field_template_string(["{{name}}", separator, "{{native_type}}"].concat())?;
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
