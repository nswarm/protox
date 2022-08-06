use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use prost_types::{FileDescriptorProto, FileDescriptorSet};

pub use renderer_config::RendererConfig;

use crate::render::Render;
use crate::renderer::context::{FileContext, MetadataContext};
use crate::{util, DisplayNormalized};

mod case;
mod context;
mod option_key_value;
mod primitive;
mod proto;
mod renderer_config;
pub mod scripted;
pub mod template;

pub const CONFIG_FILE_NAME: &'static str = "config.json";

const DEFAULT_GENERATED_HEADER: &str = r#"/////////////////////////////////////////////////////
// *** DO NOT EDIT MANUALLY ***
// This file is generated by the utility `protox`.
/////////////////////////////////////////////////////

"#;

// Delegate public Render impl to internal Renderer impl.
impl<R: Renderer> Render for R {
    fn load(&mut self, input_root: &Path) -> Result<()> {
        Renderer::load(self, input_root)
    }
    fn reset(&mut self) {
        Renderer::reset(self)
    }
    fn render(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> Result<()> {
        if self.config().one_file_per_package {
            let package_files = self.render_files_collapsed(descriptor_set, output_path)?;
            self.render_metadata_with_package_files(output_path, package_files)?;
        } else {
            self.render_files(descriptor_set, output_path)?;
            self.render_metadata_for_directories(descriptor_set, output_path)?;
        }
        Ok(())
    }
}

pub trait Renderer {
    fn load_config(path: &Path) -> Result<RendererConfig> {
        info!("Loading config from: {}", path.display_normalized());
        let file = fs::File::open(path).context("Failed to read RendererConfig file.")?;
        let buf_reader = io::BufReader::new(file);
        match path
            .extension()
            .ok_or(anyhow!("Config file must have an extension."))
        {
            Err(err) => return Err(err),
            Ok(x) => match x.to_str() {
                None => return Err(anyhow!("Config file must have an extension.")),
                Some("json") => serde_json::from_reader(buf_reader)
                    .with_context(|| error_deserialize_config("json", &path)),
                Some("yaml" | "yml") => serde_yaml::from_reader(buf_reader)
                    .with_context(|| error_deserialize_config("yaml", &path)),
                Some(x) => {
                    return Err(anyhow!(
                        "Unsupported config file type '{}'. Must be yaml, yml, or json",
                        x
                    ))
                }
            },
        }
    }

    /// Load any necessary files from the `input_root` directory.
    fn load(&mut self, input_root: &Path) -> Result<()>;

    /// Reset is called between runs with different input/outputs.
    fn reset(&mut self);

    fn config(&self) -> &RendererConfig;
    fn has_metadata(&self) -> bool;
    fn render_metadata<W: io::Write>(&self, context: MetadataContext, writer: &mut W)
        -> Result<()>;
    fn render_file<W: io::Write>(&self, context: FileContext, writer: &mut W) -> Result<()>;

    fn output_ext(&self) -> &str {
        &self.config().file_extension
    }

    fn metadata_file_name(&self) -> &str {
        &self.config().metadata_file_name
    }

    fn render_files(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> Result<()> {
        for file in &descriptor_set.file {
            if self.is_ignored_file(file) {
                log_ignore_file(&file.name, &self.config().file_extension);
                continue;
            }
            let file_name = &file_name(file, self.output_ext())?;
            info!("Rendering file for descriptor '{}'", file_name);
            let path = &output_path.join(file_name);
            let mut writer = self.file_writer(&path)?;
            log_render_file(&file.name, &self.config().file_extension);
            let context = FileContext::new(file, &self.config())?;
            self.render_file(context, &mut writer)?;
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
            let files = files
                .into_iter()
                .filter(|f| !self.is_ignored_file(f))
                .collect::<Vec<&FileDescriptorProto>>();
            if files.is_empty() {
                continue;
            }
            let path = &self.package_to_file_path(output_path, package);
            let mut writer = self.file_writer(&path)?;
            for file in files {
                log_render_package_file(file, package);
                let context = FileContext::new(file, &self.config())?;
                self.render_file(context, &mut writer)?;
            }
            package_files.insert(
                package.to_owned(),
                path.strip_prefix(output_path)?.to_path_buf(),
            );
        }
        Ok(package_files)
    }

    fn render_metadata_for_directories(
        &self,
        descriptor_set: &FileDescriptorSet,
        output_path: &Path,
    ) -> Result<()> {
        if !self.has_metadata() {
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
        let mut writer = self.file_writer(&file_path)?;
        self.render_metadata(context, &mut writer)?;
        Ok(())
    }

    fn render_metadata_with_package_files(
        &self,
        output_path: &Path,
        package_files: HashMap<String, PathBuf>,
    ) -> Result<()> {
        if !self.has_metadata() {
            return Ok(());
        }
        let mut context = MetadataContext::new();
        context.append_package_files(package_files);
        self.render_metadata_to_file(output_path, context)?;
        Ok(())
    }

    fn file_writer(&self, path: &Path) -> Result<io::BufWriter<fs::File>> {
        let path = self.config().case_config.file_name.rename_file_name(path);
        let mut writer = io::BufWriter::new(util::create_file_or_error(&path)?);
        self.write_generated_header(&mut writer)?;
        Ok(writer)
    }

    fn write_generated_header<W: io::Write>(&self, writer: &mut W) -> Result<()> {
        if let Some(configured_header) = &self.config().generated_header {
            if !configured_header.is_empty() {
                let mut header = configured_header.join("\n");
                header.push('\n');
                writer.write(header.as_bytes())?;
            }
        } else {
            writer.write(DEFAULT_GENERATED_HEADER.as_bytes())?;
        }
        Ok(())
    }

    fn collect_package_to_file_map<'a>(
        &'a self,
        descriptor_set: &'a FileDescriptorSet,
    ) -> HashMap<&'a str, Vec<&'a FileDescriptorProto>> {
        let mut map = HashMap::new();
        for file in &descriptor_set.file {
            let package = package(file, &self.config().default_package_file_name);
            let files = map.entry(package).or_insert(Vec::new());
            files.push(file);
        }
        map
    }

    fn package_to_file_path(&self, root: &Path, package: &str) -> PathBuf {
        root.join(package.replace(proto::PACKAGE_SEPARATOR, "_"))
            .with_extension(&self.config().file_extension)
    }

    fn metadata_file_path(&self, output: &Path, context: &MetadataContext) -> PathBuf {
        output
            .join(context.relative_dir())
            .join(self.metadata_file_name())
            .with_extension(&self.config().file_extension)
    }

    fn is_ignored_file(&self, file: &FileDescriptorProto) -> bool {
        match file.name.as_ref() {
            None => true,
            Some(file) => self.config().ignored_files.contains(file),
        }
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
            "Descriptor set file is missing a file name. The descriptor set was probably generated incorrectly.".to_owned()
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

fn log_render_file(file_name: &Option<String>, ext: &str) {
    debug!(
        "Rendering file: {}",
        util::replace_proto_ext(util::str_or_unknown(file_name), ext)
    );
}
fn log_ignore_file(file_name: &Option<String>, ext: &str) {
    debug!(
        "Ignoring file because it is on the ignore list: {}",
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

fn error_deserialize_config(format: &str, path: &Path) -> String {
    format!(
        "Failed to deserialize RendererConfig as {}, path: {}",
        format,
        path.display_normalized()
    )
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use prost_types::{
        DescriptorProto, EnumDescriptorProto, FieldDescriptorProto, FileDescriptorProto,
    };

    use crate::renderer::context::{FileContext, MetadataContext};
    use crate::renderer::{Renderer, RendererConfig};

    mod load_config {
        use crate::renderer::tests::FakeRenderer;
        use crate::renderer::{Renderer, RendererConfig};
        use anyhow::Result;
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        #[test]
        fn json() -> Result<()> {
            run_test("config.json", &serde_json::to_string(&config())?)
        }

        #[test]
        fn yaml() -> Result<()> {
            run_test("config.yaml", &serde_yaml::to_string(&config())?)
        }

        #[test]
        fn yml() -> Result<()> {
            run_test("config.yml", &serde_yaml::to_string(&config())?)
        }

        fn config() -> RendererConfig {
            RendererConfig {
                file_extension: "rawr".to_owned(),
                ..Default::default()
            }
        }

        fn run_test(file_name: &str, content: &str) -> Result<()> {
            let test_dir = tempdir()?;
            let config_file_path = test_dir.path().join(file_name);
            File::create(&config_file_path)?.write_all(content.as_bytes())?;

            let config = FakeRenderer::load_config(&config_file_path)?;
            assert_eq!(config.file_extension, "rawr");

            Ok(())
        }
    }

    mod render {
        use anyhow::Result;
        use prost_types::FileDescriptorSet;
        use tempfile::tempdir;

        use crate::render::Render;
        use crate::renderer::case::Case;
        use crate::renderer::tests::{fake_file_empty, fake_file_with_package, FakeRenderer};
        use crate::renderer::RendererConfig;

        #[test]
        fn render_files() -> Result<()> {
            let mut renderer = FakeRenderer::default();
            renderer.has_metadata = true;
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
            config.default_package_file_name = "pkg-root".to_owned();
            let mut renderer = FakeRenderer::with_config(config);
            renderer.has_metadata = true;
            let test_dir = tempdir()?;
            renderer.render(&test_file_set(), test_dir.path())?;

            assert!(test_dir.path().join("pkg-root").exists());
            assert!(test_dir.path().join("test").exists());
            assert!(test_dir.path().join("test-sub").exists());
            assert!(test_dir.path().join("other-sub-inner").exists());

            assert!(test_dir.path().join("metadata").exists());
            Ok(())
        }

        #[test]
        fn renders_file_with_configured_case() -> Result<()> {
            let mut config = RendererConfig::default();
            config.case_config.file_name = Case::UpperSnake;
            let renderer = FakeRenderer::with_config(config);
            let test_dir = tempdir()?;

            let set = FileDescriptorSet {
                file: vec![fake_file_empty("fileName")],
            };
            renderer.render(&set, test_dir.path())?;

            assert!(test_dir.path().join("FILE_NAME").exists());
            Ok(())
        }

        #[test]
        fn render_files_collapsed_with_configured_case() -> Result<()> {
            let mut config = RendererConfig::default();
            config.one_file_per_package = true;
            config.default_package_file_name = "pkgRoot".to_owned();
            config.case_config.file_name = Case::UpperSnake;
            let renderer = FakeRenderer::with_config(config);
            let test_dir = tempdir()?;

            let set = FileDescriptorSet {
                file: vec![fake_file_empty("fileName")],
            };
            renderer.render(&set, test_dir.path())?;

            assert!(test_dir.path().join("PKG_ROOT").exists());
            Ok(())
        }

        #[test]
        fn does_not_render_ignored_files() -> Result<()> {
            let config = RendererConfig {
                ignored_files: vec!["file1".to_owned(), "test/sub/file4".to_owned()],
                ..Default::default()
            };
            let renderer = FakeRenderer::with_config(config);
            let test_dir = tempdir()?;
            renderer.render(&test_file_set(), test_dir.path())?;

            assert!(!test_dir.path().join("file1").exists());
            assert!(test_dir.path().join("test/file2").exists());
            assert!(test_dir.path().join("test/file3").exists());
            assert!(!test_dir.path().join("test/sub/file4").exists());
            assert!(test_dir.path().join("other/sub/inner/file5").exists());
            Ok(())
        }

        #[test]
        fn does_not_render_ignored_files_collapsed() -> Result<()> {
            let config = RendererConfig {
                one_file_per_package: true,
                default_package_file_name: "pkg-root".to_owned(),
                ignored_files: vec!["file1".to_owned(), "test/sub/file4".to_owned()],
                ..Default::default()
            };
            let renderer = FakeRenderer::with_config(config);
            let test_dir = tempdir()?;
            renderer.render(&test_file_set(), test_dir.path())?;

            assert!(
                !test_dir.path().join("pkg-root").exists(),
                "should not exist because it contains an ignored file"
            );
            assert!(test_dir.path().join("test").exists());
            assert!(
                !test_dir.path().join("test-sub").exists(),
                "should not exist because it contains an ignored file"
            );
            assert!(test_dir.path().join("other-sub-inner").exists());
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
    }

    #[test]
    fn output_ext_from_config() {
        let mut config = RendererConfig::default();
        config.file_extension = "test".to_owned();
        let renderer = FakeRenderer::with_config(config.clone());
        assert_eq!(renderer.output_ext(), config.file_extension);
    }

    mod collect_dirs_and_files {
        use std::path::PathBuf;

        use anyhow::Result;
        use prost_types::FileDescriptorSet;

        use crate::renderer::collect_dirs_and_files;
        use crate::renderer::tests::fake_file_empty;

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
        use crate::renderer::tests::FakeRenderer;
        use crate::renderer::Renderer;
        use crate::renderer::RendererConfig;

        #[test]
        fn with_config() {
            let mut config = RendererConfig::default();
            config.metadata_file_name = "test".to_owned();
            let renderer = FakeRenderer::with_config(config);
            assert_eq!(renderer.metadata_file_name(), "test");
        }
    }

    mod collect_package_to_file_map {
        use anyhow::{anyhow, Result};
        use prost_types::{FileDescriptorProto, FileDescriptorSet};

        use crate::renderer::tests::{fake_file_with_package, FakeRenderer};
        use crate::renderer::Renderer;
        use crate::renderer::RendererConfig;

        #[test]
        fn collects_unique_package_to_multiple_files() -> Result<()> {
            let mut config = RendererConfig::default();
            config.default_package_file_name = "default".to_owned();
            let renderer = FakeRenderer::default();

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
        config.file_extension = "test".to_owned();
        let renderer = FakeRenderer::with_config(config);
        assert_eq!(
            renderer.package_to_file_path(&PathBuf::from("root/path/to"), "package"),
            PathBuf::from("root/path/to/package.test"),
        );
    }

    mod generated_header {
        use std::fs;
        use std::io::Read;
        use std::path::Path;

        use anyhow::{Context, Error, Result};
        use prost_types::FileDescriptorSet;
        use tempfile::tempdir;

        use crate::render::Render;
        use crate::renderer::tests::{fake_file_with_package, FakeRenderer};
        use crate::renderer::{RendererConfig, DEFAULT_GENERATED_HEADER};

        #[test]
        fn default_in_file() -> Result<()> {
            let test_dir = tempdir()?;
            render(test_dir.path(), RendererConfig::default(), false)?;
            assert_file_has_header(&test_dir.path().join("root"), DEFAULT_GENERATED_HEADER)?;
            assert_file_has_header(
                &test_dir.path().join("sub-file-0"),
                DEFAULT_GENERATED_HEADER,
            )?;
            assert_file_has_header(
                &test_dir.path().join("sub-file-1"),
                DEFAULT_GENERATED_HEADER,
            )?;
            Ok(())
        }

        #[test]
        fn default_in_single_file_package() -> Result<()> {
            let test_dir = tempdir()?;
            let mut config = RendererConfig::default();
            config.one_file_per_package = true;
            render(test_dir.path(), config, false)?;
            assert_file_has_header(&test_dir.path().join("root"), DEFAULT_GENERATED_HEADER)?;
            assert_file_has_header(&test_dir.path().join("root-sub"), DEFAULT_GENERATED_HEADER)?;
            Ok(())
        }

        #[test]
        fn default_in_metadata() -> Result<()> {
            let test_dir = tempdir()?;
            render(test_dir.path(), RendererConfig::default(), true)?;
            assert_file_has_header(&test_dir.path().join("metadata"), DEFAULT_GENERATED_HEADER)?;
            Ok(())
        }

        #[test]
        fn configured_in_file() -> Result<()> {
            let test_dir = tempdir()?;
            let mut config = RendererConfig::default();
            config.generated_header = Some(CONFIGURED_HEADER_LINES.map(&str::to_owned).to_vec());
            render(test_dir.path(), config, false)?;
            assert_file_has_header(&test_dir.path().join("root"), CONFIGURED_HEADER)?;
            assert_file_has_header(&test_dir.path().join("sub-file-0"), CONFIGURED_HEADER)?;
            assert_file_has_header(&test_dir.path().join("sub-file-1"), CONFIGURED_HEADER)?;
            Ok(())
        }

        #[test]
        fn configured_in_single_file_package() -> Result<()> {
            let test_dir = tempdir()?;
            let mut config = RendererConfig::default();
            config.generated_header = Some(CONFIGURED_HEADER_LINES.map(&str::to_owned).to_vec());
            config.one_file_per_package = true;
            render(test_dir.path(), config, false)?;
            assert_file_has_header(&test_dir.path().join("root"), CONFIGURED_HEADER)?;
            assert_file_has_header(&test_dir.path().join("root-sub"), CONFIGURED_HEADER)?;
            Ok(())
        }

        #[test]
        fn configured_in_metadata() -> Result<()> {
            let test_dir = tempdir()?;
            let mut config = RendererConfig::default();
            config.generated_header = Some(CONFIGURED_HEADER_LINES.map(&str::to_owned).to_vec());
            render(test_dir.path(), config, true)?;
            assert_file_has_header(&test_dir.path().join("metadata"), CONFIGURED_HEADER)?;
            Ok(())
        }

        #[test]
        fn configured_empty_prints_empty() -> Result<()> {
            let test_dir = tempdir()?;
            let mut config = RendererConfig::default();
            config.generated_header = Some(Vec::new());
            render(test_dir.path(), config, true)?;
            assert_file_has_header(&test_dir.path().join("metadata"), "")?;
            Ok(())
        }

        const CONFIGURED_HEADER: &str = "configured\nheader\n";
        const CONFIGURED_HEADER_LINES: [&str; 2] = ["configured", "header"];

        fn render(path: &Path, config: RendererConfig, use_metadata: bool) -> Result<(), Error> {
            let descriptor_set = FileDescriptorSet {
                file: vec![
                    fake_file_with_package("root", "root"),
                    fake_file_with_package("sub-file-0", "root.sub"),
                    fake_file_with_package("sub-file-1", "root.sub"),
                ],
            };
            let mut renderer = FakeRenderer::with_config(config);
            renderer.has_metadata = use_metadata;
            renderer.render(&descriptor_set, path)?;
            Ok(())
        }

        fn assert_file_has_header(path: &Path, header: &str) -> Result<()> {
            let mut contents = String::new();
            fs::File::open(path)
                .context("Open file with header")?
                .read_to_string(&mut contents)?;
            assert_eq!(contents, header);
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeRenderer {
        pub config: RendererConfig,
        pub has_metadata: bool,
    }

    impl FakeRenderer {
        pub fn with_config(config: RendererConfig) -> Self {
            Self {
                config,
                ..Default::default()
            }
        }
    }

    impl Renderer for FakeRenderer {
        fn load(&mut self, _input_root: &Path) -> Result<()> {
            Ok(())
        }

        fn reset(&mut self) {}

        fn config(&self) -> &RendererConfig {
            &self.config
        }

        fn has_metadata(&self) -> bool {
            self.has_metadata
        }

        fn render_metadata<W: io::Write>(
            &self,
            _context: MetadataContext,
            _writer: &mut W,
        ) -> Result<()> {
            Ok(())
        }

        fn render_file<W: io::Write>(&self, _context: FileContext, _writer: &mut W) -> Result<()> {
            Ok(())
        }
    }

    fn fake_file_with_package(
        name: impl Into<String>,
        package: impl Into<String>,
    ) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.into()),
            package: Some(package.into()),
            ..Default::default()
        }
    }

    pub fn fake_file_empty(name: impl Into<String>) -> FileDescriptorProto {
        fake_file(name, vec![], vec![])
    }

    pub fn fake_file(
        name: impl Into<String>,
        enums: Vec<EnumDescriptorProto>,
        messages: Vec<DescriptorProto>,
    ) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.into()),
            message_type: messages,
            enum_type: enums,
            ..Default::default()
        }
    }

    pub fn fake_message(
        name: impl Into<String>,
        fields: Vec<FieldDescriptorProto>,
    ) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.into()),
            field: fields,
            ..Default::default()
        }
    }

    pub fn fake_field(
        name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.into()),
            type_name: Some(type_name.into()),
            ..Default::default()
        }
    }
}
