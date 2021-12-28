use crate::template_renderer::context::{FileContext, MetadataContext};
use crate::template_renderer::indent_helper::IndentHelper;
use crate::template_renderer::proto;
use crate::template_renderer::renderer_config::RendererConfig;
use crate::{util, DisplayNormalized};
use anyhow::{anyhow, Context, Result};
use handlebars::Handlebars;
use log::{debug, info};
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{fs, io};
use walkdir::WalkDir;

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

    pub fn new() -> Self {
        let mut hbs = Handlebars::new();
        hbs.register_helper("indent", Box::new(IndentHelper));
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
    fn load_metadata_template_string(&mut self, template: impl AsRef<str>) -> Result<()> {
        self.load_template_string(Self::METADATA_TEMPLATE_NAME, template)
    }

    #[allow(dead_code)]
    fn load_file_template_string(&mut self, template: impl AsRef<str>) -> Result<()> {
        self.load_template_string(Self::FILE_TEMPLATE_NAME, template)
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
                Some(ext) if ext == Self::TEMPLATE_EXT => {}
                _ => continue,
            };

            let template_name = match file.with_extension("").file_name() {
                None => continue,
                Some(file_name) => match file_name.to_str() {
                    None => continue,
                    Some(name) => name.to_string(),
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

    pub fn render(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> Result<()> {
        if self.config.one_file_per_package {
            let package_files = self.render_files_collapsed(descriptor_set, output_path)?;
            self.render_metadata_with_package_files(output_path, package_files)?;
        } else {
            self.render_files(descriptor_set, output_path)?;
            self.render_metadata_for_directories(descriptor_set, output_path)?;
        }
        Ok(())
    }

    fn render_files(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> Result<()> {
        for file in &descriptor_set.file {
            let file_name = file_name(file, self.output_ext())?;
            info!("Rendering file for descriptor '{}'", file_name);
            let path = output_path.join(file_name);
            let mut writer = io::BufWriter::new(util::create_file_or_error(&path)?);
            self.render_file(file, &mut writer)?;
        }
        Ok(())
    }

    fn render_files_collapsed(
        &self,
        descriptor_set: &FileDescriptorSet,
        output_path: &Path,
    ) -> Result<HashMap<String, PathBuf>> {
        let package_to_files = self.collect_package_to_file_map(descriptor_set);
        let mut package_files = HashMap::new();
        for (package, files) in package_to_files {
            let path = self.package_to_file_path(output_path, package);
            let mut writer = io::BufWriter::new(util::create_file_or_error(&path)?);
            for file in files {
                log_render_package_file(file, package);
                self.render_file(file, &mut writer)?;
            }
            package_files.insert(
                package.to_string(),
                path.strip_prefix(output_path)?.to_path_buf(),
            );
        }
        Ok(package_files)
    }

    fn collect_package_to_file_map<'a>(
        &'a self,
        descriptor_set: &'a FileDescriptorSet,
    ) -> HashMap<&'a str, Vec<&'a FileDescriptorProto>> {
        let mut map = HashMap::new();
        for file in &descriptor_set.file {
            let package = package(file, &self.config.default_package_file_name);
            let files = map.entry(package).or_insert(Vec::new());
            files.push(file);
        }
        map
    }

    fn package_to_file_path(&self, root: &Path, package: &str) -> PathBuf {
        root.join(package.replace(proto::PACKAGE_SEPARATOR, "-"))
            .with_extension(&self.config.file_extension)
    }

    fn render_metadata_for_directories(
        &self,
        descriptor_set: &FileDescriptorSet,
        output_path: &Path,
    ) -> Result<()> {
        if !self.hbs.has_template(Self::METADATA_TEMPLATE_NAME) {
            return Ok(());
        }
        let (dirs, files) = collect_dirs_and_files(descriptor_set)?;
        let mut contexts = Vec::new();
        for dir in &dirs {
            let mut context = MetadataContext::with_relative_dir(dir)?;
            context.append_subdirectories(dirs.iter())?;
            context.append_files(&files)?;
            contexts.push(context);
        }
        for context in contexts {
            self.render_metadata_to_file(output_path, context)?;
        }
        Ok(())
    }

    fn render_metadata_to_file(&self, output_path: &Path, context: MetadataContext) -> Result<()> {
        let file_path = self.metadata_file_path(output_path, &context);
        log_render_metadata(&file_path);
        let mut writer = io::BufWriter::new(util::create_file_or_error(&file_path)?);
        self.render_metadata_context(context, &mut writer)?;
        Ok(())
    }

    fn render_metadata_with_package_files(
        &self,
        output_path: &Path,
        package_files: HashMap<String, PathBuf>,
    ) -> Result<()> {
        if !self.hbs.has_template(Self::METADATA_TEMPLATE_NAME) {
            return Ok(());
        }
        let mut context = MetadataContext::new();
        context.append_package_files(package_files);
        self.render_metadata_to_file(output_path, context)?;
        Ok(())
    }

    fn metadata_file_path(&self, output: &Path, context: &MetadataContext) -> PathBuf {
        output
            .join(context.relative_dir())
            .join(self.metadata_file_name())
            .with_extension(&self.config.file_extension)
    }

    fn metadata_file_name(&self) -> &str {
        &self.config.metadata_file_name
    }

    fn render_metadata_context<W: io::Write>(
        &self,
        context: MetadataContext,
        writer: &mut W,
    ) -> Result<()> {
        self.render_to_write(Self::METADATA_TEMPLATE_NAME, &context, writer)
    }

    fn render_file<W: io::Write>(&self, file: &FileDescriptorProto, writer: &mut W) -> Result<()> {
        log_render_file(&file.name, &self.config.file_extension);
        let context = FileContext::new(file, &self.config)?;
        self.render_to_write(Self::FILE_TEMPLATE_NAME, &context, writer)
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

fn collect_dirs_and_files(
    descriptor_set: &FileDescriptorSet,
) -> Result<(HashSet<PathBuf>, Vec<PathBuf>)> {
    let mut dirs = HashSet::new();
    let mut files = Vec::new();
    for file in &descriptor_set.file {
        let relative_path = file_relative_path(file)?;
        insert_all_parents(&mut dirs, &relative_path)?;
        files.push(relative_path);
    }
    Ok((dirs, files))
}

fn insert_all_parents(dirs: &mut HashSet<PathBuf>, path: &Path) -> Result<()> {
    let parent = util::path_parent_or_error(&path).context("insert_all_parents")?;
    dirs.insert(parent.to_path_buf());
    if !parent.as_os_str().is_empty() {
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

fn file_relative_path(file: &FileDescriptorProto) -> Result<PathBuf> {
    let path = PathBuf::from(file.name.as_ref().ok_or(anyhow!(
        "No file name in descriptor to create relative path from."
    ))?);
    Ok(path)
}

fn package<'a>(file: &'a FileDescriptorProto, default: &'a String) -> &'a str {
    file.package.as_ref().unwrap_or(&default)
}

fn render_error_context<S: Serialize>(name: &str, data: &S) -> String {
    format!(
        "Failed to render template '{}' for data: {}",
        name,
        serde_json::to_string(data).unwrap_or("(failed to serialize)".to_string()),
    )
}

fn log_render_file(file_name: &Option<String>, ext: &str) {
    debug!(
        "Rendering file: {}",
        util::replace_proto_ext(util::str_or_unknown(file_name), ext)
    );
}

fn log_render_package_file(file: &FileDescriptorProto, package: &str) {
    info!(
        "Rendering descriptor '{}' to file for package '{}'",
        util::str_or_unknown(&file.name),
        package,
    );
}

fn log_render_metadata(file_path: &Path) {
    info!(
        "Rendering metadata file: '{}'",
        file_path.display_normalized()
    );
}

#[cfg(test)]
mod tests {
    use crate::template_renderer::context::{
        EnumContext, FieldContext, FileContext, MessageContext,
    };
    use crate::template_renderer::primitive;
    use crate::template_renderer::renderer::Renderer;
    use crate::template_renderer::renderer_config::RendererConfig;
    use anyhow::Result;
    use prost_types::{
        DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
        FileDescriptorProto,
    };
    use std::path::PathBuf;

    mod render {
        use crate::template_renderer::renderer::tests::{fake_file_empty, fake_file_with_package};
        use crate::template_renderer::renderer::Renderer;
        use crate::template_renderer::renderer_config::RendererConfig;
        use anyhow::Result;
        use prost_types::FileDescriptorSet;
        use tempfile::tempdir;

        #[test]
        fn render_files() -> Result<()> {
            let renderer = renderer_with_templates(RendererConfig::default())?;
            let test_dir = tempdir()?;
            renderer.render(&test_file_set(), test_dir.path())?;

            assert!(test_dir.path().join("file1").exists());
            assert!(test_dir.path().join("test/file2").exists());
            assert!(test_dir.path().join("test/file3").exists());
            assert!(test_dir.path().join("test/sub/file4").exists());
            assert!(test_dir.path().join("other/sub/inner/file5").exists());

            assert!(test_dir.path().join("metadata").exists());
            assert!(test_dir.path().join("test/metadata").exists());
            assert!(test_dir.path().join("test/sub/metadata").exists());
            assert!(test_dir.path().join("other/sub/inner/metadata").exists());
            Ok(())
        }

        #[test]
        fn render_files_collapsed() -> Result<()> {
            let mut config = RendererConfig::default();
            config.one_file_per_package = true;
            config.default_package_file_name = "pkg_root".to_string();
            let renderer = renderer_with_templates(config)?;
            let test_dir = tempdir()?;
            renderer.render(&test_file_set(), test_dir.path())?;

            assert!(test_dir.path().join("pkg_root").exists());
            assert!(test_dir.path().join("test").exists());
            assert!(test_dir.path().join("test-sub").exists());
            assert!(test_dir.path().join("other-sub-inner").exists());

            assert!(test_dir.path().join("metadata").exists());
            Ok(())
        }

        fn test_file_set() -> FileDescriptorSet {
            FileDescriptorSet {
                file: vec![
                    fake_file_empty("file1"), // no package
                    fake_file_with_package("test/file2", "test"),
                    fake_file_with_package("test/file3", "test"),
                    fake_file_with_package("test/sub/file4", "test.sub"),
                    fake_file_with_package("other/sub/inner/file5", "other.sub.inner"),
                ],
            }
        }

        fn renderer_with_templates(config: RendererConfig) -> Result<Renderer<'static>> {
            let mut renderer = Renderer::with_config(config);
            renderer.load_file_template_string("{{name}}")?;
            renderer.load_metadata_template_string("metadata")?;
            Ok(renderer)
        }
    }

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
        renderer
            .load_file_template_string("{{source_file}}{{#each enums}}{{> enum}}{{/each}}{{#each messages}}{{>message}}{{/each}}")?;
        load_enum_template(&mut renderer, "{{name}}")?;
        load_message_template(&mut renderer, "{{name}}")?;
        load_message_template(&mut renderer, "{{name}}")?;

        let file_name = "file_name".to_string();
        let enum0 = fake_enum::<&str, &str>("enum0", &[]);
        let msg0 = fake_message("msg0", Vec::new());
        let msg1 = fake_message("msg1", Vec::new());
        let enum0_rendered = render_enum(&mut renderer, &enum0)?;
        let msg0_rendered = render_message(&mut renderer, &msg0)?;
        let msg1_rendered = render_message(&mut renderer, &msg1)?;
        let file = fake_file(&file_name, vec![enum0], vec![msg0, msg1]);

        let mut bytes = Vec::<u8>::new();
        renderer.render_file(&file, &mut bytes)?;

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
        let mut renderer = Renderer::with_config(config);
        renderer.load_file_template_string("{{#each imports}}{{> import}}{{/each}}")?;
        load_import_template_string(
            &mut renderer,
            "{{file_path}}{{file_name}}{{file_name_with_ext}}",
        )?;

        let file_name = "file_name".to_string();
        let mut file = fake_file_empty(&file_name);
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
        load_message_template(
            &mut renderer,
            "{{name}}{{#each fields}}{{> field}}{{/each}}".to_string(),
        )?;
        load_field_template(&mut renderer, "{{name}}:::{{native_type}}".to_string())?;

        let msg_name = "msg_name".to_string();
        let field0 = fake_field("field0", primitive::FLOAT);
        let field1 = fake_field("field1", primitive::BOOL);
        let field0_rendered = render_field(&renderer, &field0, None)?;
        let field1_rendered = render_field(&renderer, &field1, None)?;
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
        let field_name = "field-name";
        let type_name = ["TEST-", primitive::FLOAT].concat();
        let separator = ":::";
        let mut config = RendererConfig::default();
        config
            .type_config
            .insert(primitive::FLOAT.to_string(), type_name.clone());
        let mut renderer = Renderer::with_config(config);
        load_field_template(
            &mut renderer,
            ["{{field_name}}", separator, "{{fully_qualified_type}}"].concat(),
        )?;

        let field = fake_field("field-name", primitive::FLOAT);
        let result = render_field(&mut renderer, &field, Some(&".test.package".to_string()))?;
        assert_eq!(result, [field_name, separator, &type_name].concat());
        Ok(())
    }

    #[test]
    fn field_gets_package_from_file() -> Result<()> {
        let config = RendererConfig::default();
        let mut renderer = Renderer::with_config(config);
        renderer.load_file_template_string("{{#each messages}}{{> message}}{{/each}}")?;
        load_message_template(
            &mut renderer,
            "{{#each fields}}{{> field}}{{/each}}".to_string(),
        )?;
        load_field_template(&mut renderer, "{{relative_type}}".to_string())?;

        let field = fake_field("field-name", ".test.package.inner.TypeName");
        let message = fake_message("msg-name", vec![field]);
        let mut file = fake_file("file-name", vec![], vec![message]);
        file.package = Some(".test.package".to_string());
        let file_context = FileContext::new(&file, &renderer.config)?;

        let result = renderer.render_to_string(Renderer::FILE_TEMPLATE_NAME, &file_context)?;
        assert_eq!(result, "inner.TypeName");
        Ok(())
    }

    mod metadata {
        use std::collections::HashSet;
        use std::io;
        use std::iter::FromIterator;
        use std::path::PathBuf;

        use anyhow::{Context, Result};

        use crate::template_renderer::context::MetadataContext;
        use crate::template_renderer::renderer::Renderer;
        use crate::template_renderer::renderer_config::RendererConfig;

        #[test]
        fn directory() -> Result<()> {
            let directory = "directory/path";
            let mut renderer = Renderer::with_config(RendererConfig::default());
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
            let mut renderer = Renderer::with_config(RendererConfig::default());
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
            let mut renderer = Renderer::with_config(RendererConfig::default());
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
            renderer: &mut Renderer,
            dirs: HashSet<PathBuf>,
            files: Vec<PathBuf>,
            mut context: MetadataContext,
        ) -> Result<String> {
            let mut bytes = Vec::new();
            {
                let mut writer = io::BufWriter::new(&mut bytes);
                context.append_subdirectories(dirs.into_iter())?;
                context.append_files(&files)?;
                renderer.render_metadata_context(context, &mut writer)?;
            }
            Ok(String::from_utf8(bytes)
                .context("Failed to parse rendered metadata as utf8 string.")?)
        }
    }

    mod collect_dirs_and_files {
        use crate::template_renderer::renderer::collect_dirs_and_files;
        use crate::template_renderer::renderer::tests::fake_file_empty;
        use anyhow::Result;
        use prost_types::FileDescriptorSet;
        use std::path::PathBuf;

        #[test]
        fn files() -> Result<()> {
            let set = FileDescriptorSet {
                file: vec![
                    fake_file_empty("file1"),
                    fake_file_empty("test/file2"),
                    fake_file_empty("test/sub/file3"),
                    fake_file_empty("other/sub/inner/file4"),
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
                    fake_file_empty("file1"),
                    fake_file_empty("test/file2"),
                    fake_file_empty("test/sub/file3"),
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
                file: vec![fake_file_empty("test/sub/inner/file4")],
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
                    fake_file_empty("test/file1"),
                    fake_file_empty("test/file2"),
                    fake_file_empty("test/file3"),
                ],
            };
            let (dirs, _) = collect_dirs_and_files(&set)?;
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

    mod collect_package_to_file_map {
        use crate::template_renderer::renderer::tests::fake_file_with_package;
        use crate::template_renderer::renderer::Renderer;
        use crate::template_renderer::renderer_config::RendererConfig;
        use anyhow::{anyhow, Result};
        use prost_types::{FileDescriptorProto, FileDescriptorSet};

        #[test]
        fn collects_unique_package_to_multiple_files() -> Result<()> {
            let mut config = RendererConfig::default();
            config.default_package_file_name = "default".to_string();
            let renderer = Renderer::with_config(RendererConfig::default());

            let descriptor_set = FileDescriptorSet {
                file: vec![
                    fake_file_with_package("file0", "root"),
                    fake_file_with_package("file1", "root"),
                    fake_file_with_package("file2", "root.sub"),
                    fake_file_with_package("file3", "other.sub"),
                ],
            };

            let package_to_files = renderer.collect_package_to_file_map(&descriptor_set);
            let files = package_to_files
                .get("root")
                .ok_or(anyhow!("missing root"))?;
            check_has_file(files, "file0");
            check_has_file(files, "file1");

            let files = package_to_files
                .get("root.sub")
                .ok_or(anyhow!("missing root.sub"))?;
            check_has_file(files, "file2");

            let files = package_to_files
                .get("other.sub")
                .ok_or(anyhow!("missing other.sub"))?;
            check_has_file(files, "file3");
            Ok(())
        }

        fn check_has_file(files: &Vec<&FileDescriptorProto>, name: &str) {
            assert!(files
                .iter()
                .find(|f| f.name.as_ref().map(String::as_str) == Some(name))
                .is_some());
        }
    }

    #[test]
    fn package_to_file_path() {
        let mut config = RendererConfig::default();
        config.file_extension = "test".to_string();
        let renderer = Renderer::with_config(config);
        assert_eq!(
            renderer.package_to_file_path(&PathBuf::from("root/path/to"), "package"),
            PathBuf::from("root/path/to/package.test"),
        );
    }

    fn fake_file_with_package(
        name: impl Into<String>,
        package: impl Into<String>,
    ) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.into()),
            package: Some(package.into()),
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

    fn fake_file_empty(name: impl Into<String>) -> FileDescriptorProto {
        fake_file(name, vec![], vec![])
    }

    fn fake_file(
        name: impl Into<String>,
        enums: Vec<EnumDescriptorProto>,
        messages: Vec<DescriptorProto>,
    ) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.into()),
            package: None,
            dependency: vec![],
            public_dependency: vec![],
            weak_dependency: vec![],
            message_type: messages,
            enum_type: enums,
            service: vec![],
            extension: vec![],
            options: None,
            source_code_info: None,
            syntax: None,
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

    const IMPORT_TEMPLATE_NAME: &str = "import";
    const ENUM_TEMPLATE_NAME: &str = "enum";
    const MESSAGE_TEMPLATE_NAME: &str = "message";
    const FIELD_TEMPLATE_NAME: &str = "field";

    fn load_import_template_string(
        renderer: &mut Renderer,
        template: impl AsRef<str>,
    ) -> Result<()> {
        renderer.load_template_string(IMPORT_TEMPLATE_NAME, template)
    }

    fn load_enum_template(renderer: &mut Renderer, template: impl AsRef<str>) -> Result<()> {
        renderer.load_template_string(ENUM_TEMPLATE_NAME, template)
    }

    fn load_message_template(renderer: &mut Renderer, template: impl AsRef<str>) -> Result<()> {
        renderer.load_template_string(MESSAGE_TEMPLATE_NAME, template)
    }

    fn load_field_template(renderer: &mut Renderer, template: impl AsRef<str>) -> Result<()> {
        renderer.load_template_string(FIELD_TEMPLATE_NAME, template)
    }

    fn render_enum(renderer: &mut Renderer, enum_proto: &EnumDescriptorProto) -> Result<String> {
        renderer.render_to_string(
            ENUM_TEMPLATE_NAME,
            &EnumContext::new(&enum_proto, &renderer.config)?,
        )
    }

    fn render_message(renderer: &mut Renderer, message: &DescriptorProto) -> Result<String> {
        renderer.render_to_string(
            MESSAGE_TEMPLATE_NAME,
            &MessageContext::new(&message, None, &renderer.config)?,
        )
    }

    fn render_field(
        renderer: &Renderer,
        field: &FieldDescriptorProto,
        package: Option<&String>,
    ) -> Result<String> {
        let context = FieldContext::new(field, package, &renderer.config)?;
        renderer.render_to_string(FIELD_TEMPLATE_NAME, &context)
    }
}
