use crate::generator::context::RenderedField;
use crate::generator::template_config::TemplateConfig;
use crate::util;
use anyhow::Result;
use prost_types::FileDescriptorProto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileContext<'a> {
    name: &'a str,

    // Must be rendered and supplied externally.
    pub messages: Vec<RenderedField>,
}

impl<'a> FileContext<'a> {
    pub fn new(file: &'a FileDescriptorProto, _config: &TemplateConfig) -> Result<Self> {
        let context = Self {
            name: name(file)?,
            messages: Vec::new(),
        };
        Ok(context)
    }
}

fn name(file: &FileDescriptorProto) -> Result<&str> {
    util::str_or_error(&file.name, || "File has no 'name'".to_string())
}

#[cfg(test)]
mod tests {
    use crate::generator::context::FileContext;
    use crate::generator::template_config::TemplateConfig;
    use anyhow::Result;
    use prost_types::FileDescriptorProto;

    #[test]
    fn name() -> Result<()> {
        let config = TemplateConfig::default();
        let name = "file_name".to_string();
        let mut file = default_file();
        file.name = Some(name.clone());
        let context = FileContext::new(&file, &config)?;
        assert_eq!(context.name, name);
        Ok(())
    }

    #[test]
    fn missing_name_errors() {
        let config = TemplateConfig::default();
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
