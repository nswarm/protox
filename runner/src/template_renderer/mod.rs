mod context;
mod primitive;
mod renderer;
mod renderer_config;

use crate::template_config::TemplateConfig;
use crate::template_renderer::renderer::Renderer;
use crate::{util, Config, DisplayNormalized};
use anyhow::{Context, Result};
use log::info;
use prost::Message;
use prost_types::FileDescriptorSet;
use std::fs;

pub fn generate(config: &Config) -> Result<()> {
    if config.templates.is_empty() {
        return Ok(());
    }
    let descriptor_set = load_descriptor_set(&config)?;
    generate_from_descriptor_set(config, &descriptor_set)?;
    Ok(())
}

fn generate_from_descriptor_set(config: &Config, descriptor_set: &FileDescriptorSet) -> Result<()> {
    if config.templates.is_empty() {
        return Ok(());
    }
    let mut renderer = Renderer::new();
    for config in &config.templates {
        log_template_start(config);
        renderer.load_all(&config.input)?;
        util::create_dir_or_error(&config.output)?;
        renderer.render(&descriptor_set, &config.output)?;
    }
    Ok(())
}

fn log_template_start(config: &TemplateConfig) {
    info!(
        "Rendering using templates in '{}' to output directory '{}'",
        config.input.display_normalized(),
        config.output.display_normalized(),
    );
}

fn load_descriptor_set(config: &Config) -> Result<FileDescriptorSet> {
    let path = &config.descriptor_set_path;
    let bytes = fs::read(&path).with_context(|| {
        format!(
            "Failed to read file descriptor set at path: {}",
            path.display_normalized()
        )
    })?;
    let descriptor_set = Message::decode(bytes.as_slice())?;
    Ok(descriptor_set)
}

#[cfg(test)]
mod tests {
    use crate::template_config::TemplateConfig;
    use crate::template_renderer::renderer::Renderer;
    use crate::template_renderer::renderer_config::RendererConfig;
    use crate::template_renderer::{generate, generate_from_descriptor_set};
    use crate::Config;
    use anyhow::Result;
    use prost_types::{FileDescriptorProto, FileDescriptorSet};
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn no_templates_arg_is_ok() {
        let config = Config::default();
        assert!(generate(&config).is_ok());
    }

    #[test]
    fn renders_output_for_each_template_set() -> Result<()> {
        let test_dir = tempdir()?;
        let templates = ["test0", "test1", "test2"];
        let template_root = test_dir.path().join("templates");
        for template in templates {
            create_required_template_files(&template_root.join(&template))?;
        }
        let output_dir = test_dir.path().join("output");
        fs::create_dir_all(&output_dir)?;

        let mut config = Config::default();
        config.template_root = Some(template_root.clone());
        config.output_root = Some(output_dir.clone());
        for template in templates {
            config.templates.push(TemplateConfig {
                input: template_root.join(template),
                output: output_dir.join(template),
            });
        }

        let descriptor_set = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("test.proto".to_string()),
                package: None,
                dependency: vec![],
                public_dependency: vec![],
                weak_dependency: vec![],
                message_type: vec![],
                enum_type: vec![],
                service: vec![],
                extension: vec![],
                options: None,
                source_code_info: None,
                syntax: None,
            }],
        };
        generate_from_descriptor_set(&config, &descriptor_set)?;

        for template in templates {
            assert_ne!(fs::read_dir(output_dir.join(template))?.count(), 0);
        }
        Ok(())
    }

    fn create_required_template_files(path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        let mut config_json = fs::File::create(path.join(Renderer::CONFIG_FILE_NAME))?;
        config_json.write_all(serde_json::to_string(&RendererConfig::default())?.as_bytes())?;
        fs::File::create(
            path.join(Renderer::FILE_TEMPLATE_NAME)
                .with_extension(Renderer::TEMPLATE_EXT),
        )?;
        fs::File::create(
            path.join(Renderer::MESSAGE_TEMPLATE_NAME)
                .with_extension(Renderer::TEMPLATE_EXT),
        )?;
        fs::File::create(
            path.join(Renderer::FIELD_TEMPLATE_NAME)
                .with_extension(Renderer::TEMPLATE_EXT),
        )?;
        Ok(())
    }
}
