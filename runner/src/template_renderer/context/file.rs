use crate::template_renderer::context::RenderedField;
use crate::template_renderer::renderer_config::RendererConfig;
use crate::util;
use anyhow::Result;
use prost_types::FileDescriptorProto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileContext<'a> {
    source_file: &'a str,

    // Must be rendered and supplied externally.
    pub messages: Vec<RenderedField>,
}

impl<'a> FileContext<'a> {
    pub fn new(file: &'a FileDescriptorProto, _config: &RendererConfig) -> Result<Self> {
        let context = Self {
            source_file: source_file(file)?,
            messages: Vec::new(),
        };
        Ok(context)
    }
}

fn source_file(file: &FileDescriptorProto) -> Result<&str> {
    util::str_or_error(&file.name, || "File has no 'name'".to_string())
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
