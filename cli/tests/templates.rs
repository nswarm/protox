mod util;

use anyhow::Result;
use runner::generator::TemplateConfig;
use runner::Lang;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use tempfile::tempdir_in;

#[test]
fn config_file_extension() -> Result<()> {
    let test_dir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    let output_dir = "output";
    let mut config = TemplateConfig::default();
    config.file_extension = "TEST".to_string();
    test_templates(
        &config,
        test_dir.path(),
        &["--direct", &lang_with_output(&Lang::CSharp, output_dir)],
    )?;
    assert_ne!(
        fs::read_dir(test_dir.path().join(output_dir))?
            .filter_map(|entry| entry.ok())
            .filter(|e| has_ext(e, &config.file_extension))
            .count(),
        0
    );
    Ok(())
}

fn has_ext(entry: &DirEntry, expected_ext: &str) -> bool {
    match entry.path().extension() {
        None => false,
        Some(ext) => {
            println!("ext: {}", entry.path().to_str().unwrap());
            ext == expected_ext
        }
    }
}

fn lang_with_output(lang: &Lang, value: &str) -> String {
    [&lang.as_config(), "=", value].concat()
}

fn test_templates(
    config: &TemplateConfig,
    test_dir: &Path,
    additional_args: &[&str],
) -> Result<()> {
    // We copy inputs to test dir
    let tmp_inputs = test_dir.join("inputs");
    util::copy_all_resources(&tmp_inputs)?;
    util::write_config(&tmp_inputs, &config)?;
    let template_root = util::path_to_str(&tmp_inputs)?;
    let mut args = vec!["--template-root", &template_root];
    for additional_arg in additional_args {
        args.push(additional_arg);
    }
    util::test_with_args_in(test_dir, &args)?;
    Ok(())
}
