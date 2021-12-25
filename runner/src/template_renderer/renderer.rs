use crate::template_renderer::context::{
    FieldContext, FileContext, ImportContext, MessageContext, MetadataContext,
};
use crate::template_renderer::renderer_config::RendererConfig;
use crate::{util, DisplayNormalized};
use anyhow::{anyhow, Context, Result};
use handlebars::Handlebars;
use log::{debug, info};
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Renders final output files by using:
/// 1. Data from a proto descriptor set.
/// 2. A set of templates which define how the data is structured.
/// 3. A template configuration file, which supplies user choices about how to handle specifics.
pub struct Renderer<'a> {
    hbs: Handlebars<'a>,
    config: RendererConfig,
}

impl Renderer<'_> {
    pub const CONFIG_FILE_NAME: &'static str = "config.json";
    pub const TEMPLATE_EXT: &'static str = "hbs";

    pub const METADATA_TEMPLATE_NAME: &'static str = "metadata";
    pub const FILE_TEMPLATE_NAME: &'static str = "file";
    pub const IMPORT_TEMPLATE_NAME: &'static str = "import";
    pub const MESSAGE_TEMPLATE_NAME: &'static str = "message";
    pub const FIELD_TEMPLATE_NAME: &'static str = "field";

    pub fn new() -> Self {
        Self {
            hbs: Handlebars::new(),
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

    pub fn output_ext(&self) -> &str {
        &self.config.file_extension
    }

    /// Loads config and templates from the same root path with the following names:
    /// ```txt
    ///     root/config.json
    ///     root/file.hbs
    ///     root/import.hbs
    ///     root/message.hbs
    ///     root/field.hbs
    ///     root/metadata.hbs (optional)
    /// ```
    pub fn load_all(&mut self, root: &Path) -> Result<()> {
        self.load_config(&root.join(Self::CONFIG_FILE_NAME))?;
        self.load_templates(root)?;
        Ok(())
    }

    pub fn load_config(&mut self, path: &Path) -> Result<()> {
        info!("Loading config from: {}", path.display_normalized());
        let file = fs::File::open(path).context("Failed to read template config file.")?;
        let buf_reader = io::BufReader::new(file);
        self.config = serde_json::from_reader(buf_reader).with_context(|| {
            format!(
                "Failed to deserialize template config as json, path: {}",
                path.display_normalized()
            )
        })?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_metadata_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(Self::METADATA_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    pub fn load_file_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(Self::FILE_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    pub fn load_import_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(Self::IMPORT_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    pub fn load_message_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(Self::MESSAGE_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    pub fn load_field_template_string(&mut self, template: String) -> Result<()> {
        self.load_template_string(Self::FIELD_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    fn load_template_string(&mut self, name: &str, template: String) -> Result<()> {
        self.hbs
            .register_template_string(name, template)
            .with_context(|| format!("Failed to load '{}' template from string", name))?;
        Ok(())
    }

    pub fn load_templates(&mut self, root: &Path) -> Result<()> {
        self.load_metadata_template_file(&hbs_file_path(root, Self::METADATA_TEMPLATE_NAME))?;
        self.load_file_template_file(&hbs_file_path(root, Self::FILE_TEMPLATE_NAME))?;
        self.load_import_template_file(&hbs_file_path(root, Self::IMPORT_TEMPLATE_NAME))?;
        self.load_message_template_file(&hbs_file_path(root, Self::MESSAGE_TEMPLATE_NAME))?;
        self.load_field_template_file(&hbs_file_path(root, Self::FIELD_TEMPLATE_NAME))?;
        Ok(())
    }

    pub fn load_metadata_template_file(&mut self, path: &Path) -> Result<()> {
        // Metadata is optional.
        if path.exists() {
            self.load_template_file(Self::METADATA_TEMPLATE_NAME, path)
        } else {
            Ok(())
        }
    }

    pub fn load_file_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(Self::FILE_TEMPLATE_NAME, path)
    }
    pub fn load_import_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(Self::IMPORT_TEMPLATE_NAME, path)
    }

    pub fn load_message_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(Self::MESSAGE_TEMPLATE_NAME, path)
    }

    pub fn load_field_template_file(&mut self, path: &Path) -> Result<()> {
        self.load_template_file(Self::FIELD_TEMPLATE_NAME, path)
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

    pub fn render(&mut self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> Result<()> {
        self.render_files(descriptor_set, output_path)?;
        self.render_all_metadata(descriptor_set, output_path)?;
        Ok(())
    }

    fn render_files(
        &mut self,
        descriptor_set: &FileDescriptorSet,
        output_path: &Path,
    ) -> Result<()> {
        for file in &descriptor_set.file {
            let file_name = file_name(file, self.output_ext())?;
            info!("Rendering file for descriptor '{}'", file_name);
            let path = output_path.join(file_name);
            let mut writer = io::BufWriter::new(util::create_file_or_error(&path)?);
            self.render_file(file, &mut writer)?;
        }
        Ok(())
    }

    fn render_all_metadata(
        &mut self,
        descriptor_set: &FileDescriptorSet,
        output_path: &Path,
    ) -> Result<()> {
        if !self.hbs.has_template(Self::METADATA_TEMPLATE_NAME) {
            return Ok(());
        }
        let (dirs, files) = collect_dirs_and_files(descriptor_set)?;
        let mut contexts = Vec::new();
        for dir in &dirs {
            contexts.push(MetadataContext::with_relative_dir(dir)?);
        }
        for context in contexts {
            let file_path = output_path
                .join(context.relative_dir())
                .join(self.metadata_file_name())
                .with_extension(&self.config.file_extension);
            info!(
                "Rendering metadata file: '{}'",
                file_path.display_normalized()
            );
            let mut writer = io::BufWriter::new(util::create_file_or_error(&file_path)?);
            self.render_metadata_context(&dirs, &files, context, &mut writer)?;
        }
        Ok(())
    }

    fn metadata_file_name(&self) -> &str {
        &self.config.metadata_file_name
    }

    fn render_metadata_context<W: io::Write>(
        &mut self,
        dirs: &HashSet<PathBuf>,
        files: &Vec<PathBuf>,
        mut context: MetadataContext,
        writer: &mut W,
    ) -> Result<()> {
        for dir in dirs {
            context.push_subdirectory(dir)?;
        }
        for file in files {
            context.push_file(&file)?;
        }
        self.render_to_write(Self::METADATA_TEMPLATE_NAME, &context, writer)
    }

    #[allow(dead_code)]
    fn render_metadata_context_to_string(
        &mut self,
        dirs: &HashSet<PathBuf>,
        files: &Vec<PathBuf>,
        context: MetadataContext,
    ) -> Result<String> {
        let mut bytes = Vec::new();
        {
            let mut writer = io::BufWriter::new(&mut bytes);
            self.render_metadata_context(dirs, files, context, &mut writer)?;
        }
        Ok(
            String::from_utf8(bytes)
                .context("Failed to parse rendered metadata as utf8 string.")?,
        )
    }

    fn render_file<W: io::Write>(&self, file: &FileDescriptorProto, writer: &mut W) -> Result<()> {
        debug!(
            "Rendering file: {}",
            util::replace_proto_ext(
                util::str_or_unknown(&file.name),
                &self.config.file_extension
            )
        );
        let mut context = FileContext::new(file, &self.config)?;
        for import_path in &file.dependency {
            context.imports.push(self.render_import(import_path)?);
        }
        for message in &file.message_type {
            context
                .messages
                .push(self.render_message(message, file.package.as_ref())?);
        }
        self.render_to_write(Self::FILE_TEMPLATE_NAME, &context, writer)
    }

    fn render_import(&self, import_path: &String) -> Result<String> {
        debug!("Rendering import: {}", import_path);
        let context = ImportContext::new(&PathBuf::from(import_path))?;
        self.render_to_string(Self::IMPORT_TEMPLATE_NAME, &context)
    }

    fn render_message(
        &self,
        message: &DescriptorProto,
        package: Option<&String>,
    ) -> Result<String> {
        debug!("Rendering message: {}", util::str_or_unknown(&message.name));
        let mut context = MessageContext::new(message, &self.config)?;
        for field in &message.field {
            context.fields.push(self.render_field(field, package)?);
        }
        self.render_to_string(Self::MESSAGE_TEMPLATE_NAME, &context)
    }

    fn render_field(
        &self,
        field: &FieldDescriptorProto,
        package: Option<&String>,
    ) -> Result<String> {
        debug!("Rendering field: {}", util::str_or_unknown(&field.name));
        let context = FieldContext::new(field, package, &self.config)?;
        self.render_to_string(Self::FIELD_TEMPLATE_NAME, &context)
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

fn collect_dirs_and_files(
    descriptor_set: &FileDescriptorSet,
) -> Result<(HashSet<PathBuf>, Vec<PathBuf>)> {
    let mut dirs = HashSet::new();
    let mut files = Vec::new();
    for file in &descriptor_set.file {
        let relative_path = PathBuf::from(
            file.name
                .as_ref()
                .ok_or(anyhow!("No file name in descriptor."))?,
        );
        insert_all_parents(&mut dirs, &relative_path)?;
        files.push(relative_path);
    }
    Ok((dirs, files))
}

fn insert_all_parents(dirs: &mut HashSet<PathBuf>, path: &Path) -> Result<()> {
    let parent = util::path_parent_or_error(&path).context("insert_all_parents")?;
    dirs.insert(parent.to_path_buf());
    if parent != PathBuf::new() {
        insert_all_parents(dirs, parent)?;
    }
    Ok(())
}

fn file_name(file: &FileDescriptorProto, new_ext: &str) -> Result<String> {
    Ok(util::replace_proto_ext(
        util::str_or_error(&file.name, || {
            "Descriptor set file is missing a file name. The descriptor set was probably generated incorrectly.".to_string()
        })?,
        new_ext,
    ))
}

fn render_error_context<S: Serialize>(name: &str, data: &S) -> String {
    format!(
        "Failed to render template '{}' for data: {}",
        name,
        serde_json::to_string(data).unwrap_or("(failed to serialize)".to_string()),
    )
}

fn hbs_file_path(root: &Path, file_name: &str) -> PathBuf {
    let mut path = root.join(file_name);
    path.set_extension(Renderer::TEMPLATE_EXT);
    path
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::primitive;
    use crate::template_renderer::renderer::Renderer;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto};

    #[test]
    fn output_ext_from_config() {
        let mut config = RendererConfig::default();
        config.file_extension = "test".to_string();
        let renderer = Renderer::with_config(config.clone());
        assert_eq!(renderer.output_ext(), config.file_extension);
    }

    #[test]
    fn file_template() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = Renderer::with_config(config);
        renderer.load_file_template_string(
            "{{source_file}}{{#each messages}}{{this}}{{/each}}".to_string(),
        )?;
        renderer.load_message_template_string("{{name}}".to_string())?;
        renderer.load_field_template_string("{{name}}".to_string())?;

        let file_name = "file_name".to_string();
        let msg0 = fake_message("msg0", Vec::new());
        let msg1 = fake_message("msg1", Vec::new());
        let msg0_rendered = renderer.render_message(&msg0, None)?;
        let msg1_rendered = renderer.render_message(&msg1, None)?;
        let file = fake_file(&file_name, vec![msg0, msg1]);

        let mut bytes = Vec::<u8>::new();
        renderer.render_file(&file, &mut bytes)?;

        let result = String::from_utf8(bytes)?;
        assert_eq!(result, [file_name, msg0_rendered, msg1_rendered].concat());
        Ok(())
    }

    #[test]
    fn import_template() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = Renderer::with_config(config);
        renderer.load_file_template_string("{{#each imports}}{{this}}{{/each}}".to_string())?;
        renderer.load_import_template_string(
            "{{file_path}}{{file_name}}{{file_name_with_ext}}".to_string(),
        )?;
        renderer.load_message_template_string("".to_string())?;
        renderer.load_field_template_string("".to_string())?;

        let file_name = "file_name".to_string();
        let mut file = fake_file(&file_name, vec![]);
        let import0 = "root/test/value.txt".to_string();
        let import1 = "root/other/value2.rs".to_string();
        file.dependency.push(import0.clone());
        file.dependency.push(import1.clone());

        let mut bytes = Vec::<u8>::new();
        renderer.render_file(&file, &mut bytes)?;

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
        let mut renderer = Renderer::with_config(config);
        renderer.load_message_template_string(
            "{{name}}{{#each fields}}{{this}}{{/each}}".to_string(),
        )?;
        renderer.load_field_template_string("{{name}}:::{{native_type}}".to_string())?;

        let msg_name = "msg_name".to_string();
        let field0 = fake_field("field0", primitive::FLOAT);
        let field1 = fake_field("field1", primitive::BOOL);
        let field0_rendered = renderer.render_field(&field0, None)?;
        let field1_rendered = renderer.render_field(&field1, None)?;
        let message = fake_message(&msg_name, vec![field0, field1]);

        let result = renderer.render_message(&message, None)?;
        assert_eq!(
            result,
            [msg_name, field0_rendered, field1_rendered].concat()
        );
        Ok(())
    }

    #[test]
    fn field_template() -> Result<()> {
        let field_name = "field-name";
        let type_name = ["TEST-", primitive::FLOAT].concat();
        let separator = ":::";
        let mut config = RendererConfig::default();
        config
            .type_config
            .insert(primitive::FLOAT.to_string(), type_name.clone());
        let mut renderer = Renderer::with_config(config);
        renderer.load_field_template_string(
            ["{{field_name}}", separator, "{{fully_qualified_type}}"].concat(),
        )?;
        let result = renderer.render_field(
            &fake_field("field-name", primitive::FLOAT),
            Some(&".test.package".to_string()),
        )?;
        assert_eq!(result, [field_name, separator, &type_name].concat());
        Ok(())
    }

    #[test]
    fn field_gets_package_from_file() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = Renderer::with_config(config);
        renderer.load_file_template_string("{{#each messages}}{{this}}{{/each}}".to_string())?;
        renderer.load_message_template_string("{{#each fields}}{{this}}{{/each}}".to_string())?;
        renderer.load_field_template_string("{{relative_type}}".to_string())?;
        let result = renderer.render_field(
            &fake_field("field-name", ".test.package.inner.TypeName"),
            Some(&"test.package".to_string()),
        )?;
        assert_eq!(result, "inner.TypeName");
        Ok(())
    }

    mod metadata {
        use std::collections::HashSet;
        use std::iter::FromIterator;
        use std::path::PathBuf;

        use anyhow::Result;

        use crate::template_renderer::context::MetadataContext;
        use crate::template_renderer::renderer::Renderer;
        use crate::template_renderer::renderer_config::RendererConfig;

        #[test]
        fn directory() -> Result<()> {
            let directory = "directory/path";
            let mut renderer = Renderer::with_config(RendererConfig::default());
            renderer.load_metadata_template_string("{{directory}}".to_string())?;
            let result = renderer.render_metadata_context_to_string(
                &HashSet::new(),
                &Vec::new(),
                MetadataContext::with_relative_dir(&PathBuf::from(directory))?,
            )?;
            assert_eq!(result, directory);
            Ok(())
        }

        #[test]
        fn subdirectories() -> Result<()> {
            let root = PathBuf::from("root");
            let mut renderer = Renderer::with_config(RendererConfig::default());
            renderer.load_metadata_template_string(
                "{{#each subdirectories}}{{this}}{{#unless @last}}:::{{/unless}}{{/each}}"
                    .to_string(),
            )?;
            let result = renderer.render_metadata_context_to_string(
                &HashSet::from_iter(
                    vec![
                        root.join("sub0"),
                        root.join("sub1"),
                        root.join("sub1/too_deep"),
                        PathBuf::from("not_root"),
                    ]
                    .into_iter(),
                ),
                &Vec::new(),
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
            let mut renderer = Renderer::with_config(RendererConfig::default());
            renderer.load_metadata_template_string(
                "{{#each file_names_with_ext}}{{this}}{{/each}}".to_string(),
            )?;
            let result = renderer.render_metadata_context_to_string(
                &HashSet::new(),
                &vec![
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
    }

    mod collect_dirs_and_files {
        use crate::template_renderer::renderer::collect_dirs_and_files;
        use crate::template_renderer::renderer::tests::fake_file;
        use anyhow::Result;
        use prost_types::FileDescriptorSet;
        use std::path::PathBuf;

        #[test]
        fn files() -> Result<()> {
            let set = FileDescriptorSet {
                file: vec![
                    fake_file("file1", Vec::new()),
                    fake_file("test/file2", Vec::new()),
                    fake_file("test/sub/file3", Vec::new()),
                    fake_file("other/sub/inner/file4", Vec::new()),
                ],
            };
            let (_, files) = collect_dirs_and_files(&set)?;
            assert!(files.contains(&PathBuf::from("file1")));
            assert!(files.contains(&PathBuf::from("test/file2")));
            assert!(files.contains(&PathBuf::from("test/sub/file3")));
            assert!(files.contains(&PathBuf::from("other/sub/inner/file4")));
            Ok(())
        }

        #[test]
        fn directories() -> Result<()> {
            let set = FileDescriptorSet {
                file: vec![
                    fake_file("file1", Vec::new()),
                    fake_file("test/file2", Vec::new()),
                    fake_file("test/sub/file3", Vec::new()),
                ],
            };
            let (dirs, _) = collect_dirs_and_files(&set)?;
            assert!(dirs.contains(&PathBuf::new()));
            assert!(dirs.contains(&PathBuf::from("test")));
            assert!(dirs.contains(&PathBuf::from("test/sub")));
            Ok(())
        }

        #[test]
        fn includes_directories_with_no_files() -> Result<()> {
            let set = FileDescriptorSet {
                file: vec![fake_file("test/sub/inner/file4", Vec::new())],
            };
            let (dirs, _) = collect_dirs_and_files(&set)?;
            assert!(dirs.contains(&PathBuf::new()));
            assert!(dirs.contains(&PathBuf::from("test")));
            assert!(dirs.contains(&PathBuf::from("test/sub")));
            assert!(dirs.contains(&PathBuf::from("test/sub/inner")));
            Ok(())
        }

        #[test]
        fn ignores_duplicate_dirs() -> Result<()> {
            let set = FileDescriptorSet {
                file: vec![
                    fake_file("test/file1", Vec::new()),
                    fake_file("test/file2", Vec::new()),
                    fake_file("test/file3", Vec::new()),
                ],
            };
            let (dirs, _) = collect_dirs_and_files(&set)?;
            println!("{:?}", dirs);
            assert_eq!(dirs.len(), 2);
            assert!(dirs.contains(&PathBuf::new()));
            assert!(dirs.contains(&PathBuf::from("test")));
            Ok(())
        }
    }

    mod metadata_file_name {
        use crate::template_renderer::renderer::Renderer;
        use crate::template_renderer::renderer_config::RendererConfig;

        #[test]
        fn default() {
            let renderer = Renderer::with_config(RendererConfig::default());
            assert_eq!(
                renderer.metadata_file_name(),
                Renderer::METADATA_TEMPLATE_NAME
            );
        }

        #[test]
        fn with_config() {
            let mut config = RendererConfig::default();
            config.metadata_file_name = "test".to_string();
            let renderer = Renderer::with_config(config);
            assert_eq!(renderer.metadata_file_name(), "test");
        }
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
