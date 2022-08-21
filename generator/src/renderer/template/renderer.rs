use crate::renderer::context::{FileContext, MetadataContext};
use crate::renderer::template::{helper, FILE_TEMPLATE_NAME, METADATA_TEMPLATE_NAME, TEMPLATE_EXT};
use crate::renderer::{find_existing_config_path, Renderer, RendererConfig};
use crate::DisplayNormalized;
use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Renders final output files by using:
/// 1. Data from a proto descriptor set.
/// 2. A set of templates which define how the data is structured.
/// 3. A template configuration file, which supplies user choices about how to handle specifics.
pub struct TemplateRenderer<'a> {
    hbs: Handlebars<'a>,
    config: RendererConfig,
}

impl TemplateRenderer<'_> {
    pub fn new() -> Self {
        let mut hbs = Handlebars::new();
        hbs.register_helper("indent", Box::new(helper::Indent));
        hbs.register_helper("if_equals", Box::new(helper::IfEquals));
        Self {
            hbs,
            config: Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: RendererConfig) -> Self {
        Self {
            hbs: Handlebars::new(),
            config,
        }
    }

    #[allow(dead_code)]
    fn load_metadata_template_string(&mut self, template: impl AsRef<str>) -> Result<()> {
        self.load_template_string(METADATA_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    fn load_file_template_string(&mut self, template: impl AsRef<str>) -> Result<()> {
        self.load_template_string(FILE_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    fn load_template_string(&mut self, name: &str, template: impl AsRef<str>) -> Result<()> {
        self.hbs
            .register_template_string(name, template)
            .with_context(|| format!("Failed to load '{}' template from string", name))?;
        Ok(())
    }

    pub fn load_templates(&mut self, root: &Path) -> Result<()> {
        for entry in WalkDir::new(root)
            .follow_links(false)
            .max_depth(1)
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file = entry.path();
            match file.extension() {
                Some(ext) if ext == TEMPLATE_EXT => {}
                _ => continue,
            };

            let template_name = match file.file_stem() {
                None => continue,
                Some(file_name) => match file_name.to_str() {
                    None => continue,
                    Some(name) => name.to_owned(),
                },
            };

            self.load_template_file(&template_name, file)?;
        }
        Ok(())
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

    #[allow(dead_code)]
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

impl Renderer for TemplateRenderer<'_> {
    /// Loads config and templates from the same root path with the following names:
    /// ```txt
    ///     root/config.json
    ///     root/file.hbs
    ///     root/metadata.hbs (optional)
    /// ```
    ///
    /// Any other `*.hbs` files will also be loaded as templates based on the file name, and can
    /// be used in other templates as partials with the syntax {{> file_name}}.
    /// (See also: https://handlebarsjs.com/guide/partials.html)
    fn load(&mut self, root: &Path, _: &[PathBuf]) -> Result<()> {
        self.config = Self::load_config(&find_existing_config_path(root)?, &[])?;
        self.load_templates(root)?;
        Ok(())
    }

    fn reset(&mut self) {
        self.hbs.clear_templates();
    }

    fn config(&self) -> &RendererConfig {
        &self.config
    }

    fn has_metadata(&self) -> bool {
        self.hbs.has_template(METADATA_TEMPLATE_NAME)
    }

    fn render_metadata<W: io::Write>(
        &self,
        context: MetadataContext,
        writer: &mut W,
    ) -> Result<()> {
        self.render_to_write(METADATA_TEMPLATE_NAME, &context, writer)
    }

    fn render_file<W: io::Write>(&self, context: FileContext, writer: &mut W) -> Result<()> {
        self.render_to_write(FILE_TEMPLATE_NAME, &context, writer)
    }
}

fn render_error_context<S: Serialize>(name: &str, data: &S) -> String {
    format!(
        "Failed to render template '{}' for data: {}",
        name,
        serde_json::to_string(data).unwrap_or("(failed to serialize)".to_owned()),
    )
}

#[cfg(test)]
mod tests {
    use crate::renderer::context::{EnumContext, FieldContext, FileContext, MessageContext};
    use crate::renderer::template::renderer::TemplateRenderer;
    use crate::renderer::template::FILE_TEMPLATE_NAME;
    use crate::renderer::tests::{fake_field, fake_file, fake_file_empty, fake_message};
    use crate::renderer::{primitive, Renderer, RendererConfig};
    use anyhow::Result;
    use prost_types::{
        DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    };
    use std::collections::HashMap;

    #[test]
    fn file_template() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = TemplateRenderer::with_config(config.clone());
        renderer
            .load_file_template_string("{{source_file}}{{#each enums}}{{> enum}}{{/each}}{{#each messages}}{{>message}}{{/each}}")?;
        load_enum_template(&mut renderer, "{{name}}")?;
        load_message_template(&mut renderer, "{{name}}")?;
        load_message_template(&mut renderer, "{{name}}")?;

        let file_name = "file_name".to_owned();
        let enum0 = fake_enum::<&str, &str>("enum0", &[]);
        let msg0 = fake_message("msg0", Vec::new());
        let msg1 = fake_message("msg1", Vec::new());
        let enum0_rendered = render_enum(&mut renderer, &enum0)?;
        let msg0_rendered = render_message(&mut renderer, &msg0)?;
        let msg1_rendered = render_message(&mut renderer, &msg1)?;
        let file = fake_file(&file_name, vec![enum0], vec![msg0, msg1]);

        let mut bytes = Vec::<u8>::new();
        let context = FileContext::new(&file, &config)?;
        renderer.render_file(context, &mut bytes)?;

        let result = String::from_utf8(bytes)?;
        assert_eq!(
            result,
            [file_name, enum0_rendered, msg0_rendered, msg1_rendered].concat()
        );
        Ok(())
    }

    #[test]
    fn import_template() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = TemplateRenderer::with_config(config.clone());
        renderer.load_file_template_string("{{#each imports}}{{> import}}{{/each}}")?;
        load_import_template_string(
            &mut renderer,
            "{{file_path}}{{file_name}}{{file_name_with_ext}}",
        )?;

        let file_name = "file_name".to_owned();
        let mut file = fake_file_empty(&file_name);
        let import0 = "root/test/value.txt".to_owned();
        let import1 = "root/other/value2.rs".to_owned();
        file.dependency.push(import0.clone());
        file.dependency.push(import1.clone());

        let mut bytes = Vec::<u8>::new();
        let context = FileContext::new(&file, &config)?;
        renderer.render_file(context, &mut bytes)?;

        let result = String::from_utf8(bytes)?;
        assert_eq!(
            result,
            [
                &import0,
                "value",
                "value.txt",
                &import1,
                "value2",
                "value2.rs"
            ]
            .concat()
        );
        Ok(())
    }

    #[test]
    fn message_template() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = TemplateRenderer::with_config(config);
        load_message_template(
            &mut renderer,
            "{{name}}{{#each fields}}{{> field}}{{/each}}".to_owned(),
        )?;
        load_field_template(&mut renderer, "{{name}}:::{{native_type}}".to_owned())?;

        let msg_name = "MsgName".to_owned();
        let field0 = fake_field("field0", primitive::FLOAT);
        let field1 = fake_field("field1", primitive::BOOL);
        let field0_rendered = render_field(&renderer, &field0, None, None)?;
        let field1_rendered = render_field(&renderer, &field1, None, None)?;
        let message = fake_message(&msg_name, vec![field0, field1]);

        let result = render_message(&mut renderer, &message)?;
        assert_eq!(
            result,
            [msg_name, field0_rendered, field1_rendered].concat()
        );
        Ok(())
    }

    #[test]
    fn field_template() -> Result<()> {
        let field_name = "field_name";
        let type_name = ["Test", primitive::FLOAT].concat();
        let separator = ":::";
        let mut config = RendererConfig::default();
        config
            .type_config
            .insert(primitive::FLOAT.to_owned(), type_name.clone());
        let mut renderer = TemplateRenderer::with_config(config);
        load_field_template(
            &mut renderer,
            ["{{field_name}}", separator, "{{fully_qualified_type}}"].concat(),
        )?;

        let field = fake_field("field-name", primitive::FLOAT);
        let result = render_field(
            &mut renderer,
            &field,
            Some(&".test.package".to_owned()),
            None,
        )?;
        assert_eq!(result, [field_name, separator, &type_name].concat());
        Ok(())
    }

    #[test]
    fn field_gets_package_from_file() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = TemplateRenderer::with_config(config);
        renderer.load_file_template_string("{{#each messages}}{{> message}}{{/each}}")?;
        load_message_template(
            &mut renderer,
            "{{#each fields}}{{> field}}{{/each}}".to_owned(),
        )?;
        load_field_template(&mut renderer, "{{relative_type}}".to_owned())?;

        let field = fake_field("field-name", ".test.package.inner.TypeName");
        let message = fake_message("msg-name", vec![field]);
        let mut file = fake_file("file-name", vec![], vec![message]);
        file.package = Some(".test.package".to_owned());
        let file_context = FileContext::new(&file, &renderer.config)?;

        let result = renderer.render_to_string(FILE_TEMPLATE_NAME, &file_context)?;
        assert_eq!(result, "inner.TypeName");
        Ok(())
    }

    mod metadata {
        use std::collections::HashSet;
        use std::io;
        use std::iter::FromIterator;
        use std::path::PathBuf;

        use anyhow::{Context, Result};

        use crate::renderer::context::MetadataContext;
        use crate::renderer::template::renderer::TemplateRenderer;
        use crate::renderer::Renderer;

        #[test]
        fn directory() -> Result<()> {
            let directory = "directory/path";
            let mut renderer = TemplateRenderer::new();
            renderer.load_metadata_template_string("{{directory}}")?;
            let result = render_metadata_context_to_string(
                &mut renderer,
                HashSet::new(),
                Vec::new(),
                MetadataContext::with_relative_dir(&PathBuf::from(directory))?,
            )?;
            assert_eq!(result, directory);
            Ok(())
        }

        #[test]
        fn subdirectories() -> Result<()> {
            let root = PathBuf::from("root");
            let mut renderer = TemplateRenderer::new();
            renderer.load_metadata_template_string(
                "{{#each subdirectories}}{{this}}{{#unless @last}}:::{{/unless}}{{/each}}",
            )?;
            let result = render_metadata_context_to_string(
                &mut renderer,
                HashSet::from_iter(
                    vec![
                        root.join("sub0"),
                        root.join("sub1"),
                        root.join("sub1/too_deep"),
                        PathBuf::from("not_root"),
                    ]
                    .into_iter(),
                ),
                Vec::new(),
                MetadataContext::with_relative_dir(&PathBuf::from(root))?,
            )?;
            // Can't rely on order.
            let dirs = result.split(":::").collect::<Vec<&str>>();
            assert_eq!(dirs.len(), 2);
            assert!(dirs.contains(&"sub0"));
            assert!(dirs.contains(&"sub1"));
            Ok(())
        }

        #[test]
        fn files() -> Result<()> {
            let root = PathBuf::from("root");
            let mut renderer = TemplateRenderer::new();
            renderer
                .load_metadata_template_string("{{#each file_names_with_ext}}{{this}}{{/each}}")?;
            let result = render_metadata_context_to_string(
                &mut renderer,
                HashSet::new(),
                vec![
                    root.join("file.txt"),
                    root.join("_other_file.rs"),
                    root.join("sub1/too_deep.txt"),
                    PathBuf::from("not_root/file.txt"),
                ],
                MetadataContext::with_relative_dir(&PathBuf::from(root))?,
            )?;
            assert_eq!(result, "file.txt_other_file.rs");
            Ok(())
        }

        fn render_metadata_context_to_string(
            renderer: &mut impl Renderer,
            dirs: HashSet<PathBuf>,
            files: Vec<PathBuf>,
            mut context: MetadataContext,
        ) -> Result<String> {
            let mut bytes = Vec::new();
            {
                let mut writer = io::BufWriter::new(&mut bytes);
                context.append_subdirectories(dirs.into_iter())?;
                context.append_files(&files)?;
                renderer.render_metadata(context, &mut writer)?;
            }
            Ok(String::from_utf8(bytes)
                .context("Failed to parse rendered metadata as utf8 string.")?)
        }
    }

    fn fake_enum<N, V>(name: N, values: &[(&V, i32)]) -> EnumDescriptorProto
    where
        N: Into<String>,
        V: ToString,
    {
        EnumDescriptorProto {
            name: Option::<String>::Some(name.into()),
            value: values
                .into_iter()
                .map(|(name, number)| EnumValueDescriptorProto {
                    name: Some(name.to_string()),
                    number: Some(number.clone()),
                    options: None,
                })
                .collect(),
            options: None,
            reserved_range: vec![],
            reserved_name: vec![],
        }
    }

    const IMPORT_TEMPLATE_NAME: &str = "import";
    const ENUM_TEMPLATE_NAME: &str = "enum";
    const MESSAGE_TEMPLATE_NAME: &str = "message";
    const FIELD_TEMPLATE_NAME: &str = "field";

    fn load_import_template_string(
        renderer: &mut TemplateRenderer,
        template: impl AsRef<str>,
    ) -> Result<()> {
        renderer.load_template_string(IMPORT_TEMPLATE_NAME, template)
    }

    fn load_enum_template(
        renderer: &mut TemplateRenderer,
        template: impl AsRef<str>,
    ) -> Result<()> {
        renderer.load_template_string(ENUM_TEMPLATE_NAME, template)
    }

    fn load_message_template(
        renderer: &mut TemplateRenderer,
        template: impl AsRef<str>,
    ) -> Result<()> {
        renderer.load_template_string(MESSAGE_TEMPLATE_NAME, template)
    }

    fn load_field_template(
        renderer: &mut TemplateRenderer,
        template: impl AsRef<str>,
    ) -> Result<()> {
        renderer.load_template_string(FIELD_TEMPLATE_NAME, template)
    }

    fn render_enum(
        renderer: &mut TemplateRenderer,
        enum_proto: &EnumDescriptorProto,
    ) -> Result<String> {
        renderer.render_to_string(
            ENUM_TEMPLATE_NAME,
            &EnumContext::new(&enum_proto, None, &renderer.config)?,
        )
    }

    fn render_message(
        renderer: &mut TemplateRenderer,
        message: &DescriptorProto,
    ) -> Result<String> {
        renderer.render_to_string(
            MESSAGE_TEMPLATE_NAME,
            &MessageContext::new(&message, None, &renderer.config)?,
        )
    }

    fn render_field(
        renderer: &TemplateRenderer,
        field: &FieldDescriptorProto,
        package: Option<&String>,
        message_name: Option<&String>,
    ) -> Result<String> {
        let context = FieldContext::new(
            field,
            package,
            message_name,
            &HashMap::new(),
            &renderer.config,
        )?;
        renderer.render_to_string(FIELD_TEMPLATE_NAME, &context)
    }
}
