use crate::template_renderer::context::{EnumContext, ImportContext, MessageContext};
use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;
use anyhow::Result;
use log::debug;
use prost_types::FileDescriptorProto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileContext<'a> {
    source_file: &'a str,
    imports: Vec<ImportContext>,
    enums: Vec<EnumContext>,
    messages: Vec<MessageContext>,
}

impl<'a> FileContext<'a> {
    pub fn new(file: &'a FileDescriptorProto, config: &'a RendererConfig) -> Result<Self> {
        debug!(
            "Creating file context: {}",
            util::str_or_unknown(&file.name)
        );
        let context = Self {
            source_file: source_file(file)?,
            imports: imports(file)?,
            enums: enums(file, config)?,
            messages: messages(file, file.package.as_ref(), config)?,
        };
        Ok(context)
    }
}

fn source_file(file: &FileDescriptorProto) -> Result<&str> {
    util::str_or_error(&file.name, || "File has no 'name'".to_string())
}

fn imports(file: &FileDescriptorProto) -> Result<Vec<ImportContext>> {
    let mut imports = Vec::new();
    for import in &file.dependency {
        imports.push(ImportContext::new(import)?);
    }
    Ok(imports)
}

fn enums<'a>(
    file: &'a FileDescriptorProto,
    config: &'a RendererConfig,
) -> Result<Vec<EnumContext>> {
    let mut enums = Vec::new();
    for proto in &file.enum_type {
        enums.push(EnumContext::new(proto, config)?);
    }
    Ok(enums)
}

fn messages<'a>(
    file: &'a FileDescriptorProto,
    package: Option<&String>,
    config: &'a RendererConfig,
) -> Result<Vec<MessageContext>> {
    let mut messages = Vec::new();
    for message in &file.message_type {
        messages.push(MessageContext::new(message, package, config)?);
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::context::FileContext;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;
    use prost_types::FileDescriptorProto;

    #[test]
    fn source_file() -> Result<()> {
        let config = RendererConfig::default();
        let name = "file_name".to_string();
        let mut file = default_file();
        file.name = Some(name.clone());
        let context = FileContext::new(&file, &config)?;
        assert_eq!(context.source_file, name);
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = RendererConfig::default();
        let file = default_file();
        let result = FileContext::new(&file, &config);
        assert!(result.is_err());
    }

    fn default_file() -> FileDescriptorProto {
        FileDescriptorProto {
            name: None,
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
        }
    }
}
