use crate::render::Render;
use crate::{util, Config, DisplayNormalized, InOutConfig};
use anyhow::Context;
use anyhow::Result;
use log::info;
use prost_types::FileDescriptorSet;

pub trait InOutGenerator<R: Render> {
    fn name(&self) -> &str;
    fn renderer(&mut self) -> &mut R;

    fn app_config(&self) -> &Config;
    fn in_out_configs(&self) -> Vec<InOutConfig>;

    fn generate(&mut self) -> Result<()> {
        if self.in_out_configs().is_empty() {
            return Ok(());
        }
        let descriptor_set = util::load_descriptor_set(self.app_config())?;
        self.generate_from_descriptor_set(&descriptor_set)?;
        Ok(())
    }

    fn generate_from_descriptor_set(&mut self, descriptor_set: &FileDescriptorSet) -> Result<()> {
        if self.in_out_configs().is_empty() {
            return Ok(());
        }
        for config in &self.in_out_configs() {
            log_render_start(self.name(), &config);
            self.renderer().load(&config.input, &config.overlays)?;
            util::create_dir_or_error(&config.output)
                .with_context(|| error_context(self.name()))?;
            util::check_dir_is_empty(&config.output).with_context(|| error_context(self.name()))?;
            self.renderer().render(&descriptor_set, &config.output)?;
        }
        Ok(())
    }
}

fn error_context(name: &str) -> String {
    format!("InOutGenerator '{}' out dir", name)
}

fn log_render_start(name: &str, config: &InOutConfig) {
    info!(
        "Rendering using '{}' in '{}' to output directory '{}'",
        name,
        config.input.display_normalized(),
        config.output.display_normalized(),
    );
}

#[cfg(test)]
mod tests {
    use crate::in_out_generator::InOutGenerator;
    use crate::render::Render;
    use crate::{util, Config, InOutConfig};
    use anyhow::Result;
    use prost_types::{FileDescriptorProto, FileDescriptorSet};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn no_in_out_configs_is_ok() {
        assert!(TestGenerator {
            renderer: TestRenderer {},
            config: &Default::default(),
            in_out_configs: vec![]
        }
        .generate()
        .is_ok());
    }

    #[test]
    fn errors_if_output_dir_is_not_empty() -> Result<()> {
        let test_dir = tempdir()?;
        let input_dir = test_dir.path().join("input");
        let output_dir = test_dir.path().join("output");
        let descriptor_set = FileDescriptorSet { file: vec![] };
        let config_path = "test";
        let config = Config::default();
        let mut generator =
            TestGenerator::with_in_out(&config, &input_dir, &output_dir, &[config_path]);
        util::create_dir_or_error(&output_dir.join(config_path))?;
        let _ = fs::File::create(output_dir.join(config_path).join("some_file"))?;
        assert!(generator
            .generate_from_descriptor_set(&descriptor_set)
            .is_err());
        Ok(())
    }

    #[test]
    fn renders_output_for_each_in_out_set() -> Result<()> {
        let test_dir = tempdir()?;
        let input_dir = test_dir.path().join("input");
        let output_dir = test_dir.path().join("output");
        let descriptor_set = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("test.proto".to_owned()),
                ..Default::default()
            }],
        };
        let in_out = ["test0", "test1", "test2"];
        let config = Config::default();
        let mut generator = TestGenerator::with_in_out(&config, &input_dir, &output_dir, &in_out);
        generator.generate_from_descriptor_set(&descriptor_set)?;

        for path in in_out {
            assert_ne!(fs::read_dir(output_dir.join(path))?.count(), 0);
        }
        Ok(())
    }

    struct TestRenderer {}
    impl Render for TestRenderer {
        fn load(&mut self, _input_root: &Path, _overlays: &[PathBuf]) -> Result<()> {
            Ok(())
        }

        fn reset(&mut self) {}

        fn render(
            &self,
            _descriptor_set: &FileDescriptorSet,
            output_path: &Path,
        ) -> anyhow::Result<()> {
            fs::File::create(output_path.join("testfile.test"))?;
            Ok(())
        }
    }
    struct TestGenerator<'a> {
        renderer: TestRenderer,
        config: &'a Config,
        in_out_configs: Vec<InOutConfig>,
    }
    impl<'a> TestGenerator<'a> {
        fn with_in_out(config: &'a Config, input: &Path, output: &Path, paths: &[&str]) -> Self {
            Self {
                renderer: TestRenderer {},
                config: &config,
                in_out_configs: paths
                    .iter()
                    .map(|path| InOutConfig {
                        input: input.join(path),
                        output: output.join(path),
                        overlays: vec![],
                    })
                    .collect::<Vec<InOutConfig>>(),
            }
        }
    }
    impl InOutGenerator<TestRenderer> for TestGenerator<'_> {
        fn name(&self) -> &str {
            "TestGenerator"
        }

        fn renderer(&mut self) -> &mut TestRenderer {
            &mut self.renderer
        }

        fn app_config(&self) -> &Config {
            self.config
        }

        fn in_out_configs(&self) -> Vec<InOutConfig> {
            self.in_out_configs.clone()
        }
    }
}
