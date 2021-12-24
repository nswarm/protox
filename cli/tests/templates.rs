// mod util;
//
// use anyhow::Result;
// use runner::template_renderer::TemplateConfig;
// use std::fs;
// use std::fs::DirEntry;
// use std::path::{Path, PathBuf};
// use tempfile::tempdir_in;
//
// fn test_templates(
//     config: &TemplateConfig,
//     test_dir: &Path,
//     additional_args: &[&str],
// ) -> Result<PathBuf> {
//     // We copy inputs to test dir so we can overwrite the config.json.
//     let tmp_inputs = test_dir.join("inputs");
//     util::copy_all_resources(&tmp_inputs)?;
//     util::write_config(&tmp_inputs, &config)?;
//     let template_dir = util::path_to_str(&tmp_inputs)?;
//     println!("{:?}", tmp_inputs);
//     let mut args = vec!["--templates", &template_dir];
//     for additional_arg in additional_args {
//         args.push(additional_arg);
//     }
//     let output_dir = test_dir.join("outputs");
//     util::test_with_args_in(&output_dir, &args)?;
//     Ok(output_dir)
// }
