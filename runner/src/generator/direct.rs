use crate::generator::renderer::Renderer;
use crate::generator::template_config::TemplateConfig;
use crate::{util, Config, Lang};
use anyhow::{anyhow, Context, Result};
use log::info;
use prost::Message;
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub const SUPPORTED_LANGUAGES: [Lang; 1] = [Lang::CSharp];

pub fn generate(app_config: &Config) -> Result<()> {
    if app_config.direct.is_empty() {
        return Ok(());
    }
    util::create_output_dirs(&app_config.direct)?;
    let descriptor_set = load_descriptor_set(&app_config)?;
    for lang_config in &app_config.direct {
        info!(
            "Generating 'direct' for language '{}' to output path: {}",
            lang_config.lang.as_config(),
            util::normalize_slashes(lang_config.output.display()),
        );
        let output_path = &lang_config.output;
        let template_config = TemplateConfig::default(); // todo load from file.
        let renderer = Renderer::with_config(template_config);
        render_descriptor_set(&descriptor_set, output_path, renderer)?;
    }
    Ok(())
}

fn render_descriptor_set(
    descriptor_set: &FileDescriptorSet,
    output_path: &PathBuf,
    renderer: Renderer,
) -> Result<()> {
    for file in &descriptor_set.file {
        info!("Rendering file for descriptor '{}'", file_name(file)?);
        let path = output_path.join(file_name(file)?);
        match path.parent() {
            None => {}
            Some(parent) => {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create directories for path {}",
                        util::normalize_slashes(parent.display())
                    )
                })?;
            }
        }
        let mut writer = io::BufWriter::new(fs::File::create(&path).with_context(|| {
            format!(
                "Failed to create file at path {}",
                util::normalize_slashes(path.display())
            )
        })?);
        renderer.render_file(file, &mut writer)?;
    }
    Ok(())
}

fn file_name(file: &FileDescriptorProto) -> Result<&str> {
    util::str_or_error(&file.name, || {
        "Descriptor set file is missing a file name. The descriptor set was probably generated incorrectly.".to_string()
    })
}

fn load_descriptor_set(app_config: &Config) -> Result<FileDescriptorSet> {
    let path = &app_config.descriptor_set_path;
    let bytes = fs::read(&path).with_context(|| {
        format!(
            "Failed to read file descriptor set at path: {}",
            util::normalize_slashes(path.display())
        )
    })?;
    let descriptor_set = Message::decode(bytes.as_slice())?;
    Ok(descriptor_set)
}
