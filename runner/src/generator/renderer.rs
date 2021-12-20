use crate::generator::config::Config;
use crate::generator::context;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use prost_types::FieldDescriptorProto;

pub struct Renderer<'a> {
    hbs: Handlebars<'a>,
    config: Config,
}

const FIELD_TEMPLATE_NAME: &str = "field";

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
        self.hbs
            .register_template_string(FIELD_TEMPLATE_NAME, template)
            .context("Failed to load field template")?;
        Ok(())
    }

    pub fn render_field(&self, field: &FieldDescriptorProto) -> Result<String> {
        let rendered = self
            .hbs
            .render(FIELD_TEMPLATE_NAME, &context::field(field, &self.config)?)
            .with_context(|| {
                format!(
                    "Failed to render field: {}",
                    field.name.as_ref().unwrap_or(&"unknown".to_string())
                )
            })?;
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    mod field {
        use anyhow::Result;
        use prost_types::FieldDescriptorProto;

        use crate::generator::config::Config;
        use crate::generator::renderer::Renderer;

        // #[test]
        // fn configured_type() -> Result<()> {
        //     let mut config = Config::default();
        //     config
        //         .type_config
        //         .insert("float".to_string(), "TEST-float".to_string());
        //     let mut renderer = Renderer::with_config(config);
        //     renderer.load_field_template("{{name}}:::{{type_name}}".to_string())?;
        //     let result = renderer.render_field(&fake_field("field-name", "float"))?;
        //     assert_eq!(result, "field-name:::TEST-float");
        //     Ok(())
        // }

        fn fake_field(
            name: impl Into<String>,
            type_name: impl Into<String>,
        ) -> FieldDescriptorProto {
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
}
