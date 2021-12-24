mod util;

use crate::util::resources_dir;
use anyhow::Result;
use std::fs;
use tempfile::tempdir_in;

#[test]
fn test_templates() -> Result<()> {
    let test_dir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    let inputs = [
        resources_dir().join("template-a"),
        resources_dir().join("template-b"),
    ];
    let outputs = [
        test_dir.path().join("output-a"),
        test_dir.path().join("output-b"),
    ];
    let args = vec![
        "--template".to_string(),
        util::path_to_str(&inputs[0])?,
        util::path_to_str(&outputs[0])?,
        "--template".to_string(),
        util::path_to_str(&inputs[1])?,
        util::path_to_str(&outputs[1])?,
    ];

    util::test_with_args_in(test_dir.path(), &args)?;

    assert_ne!(
        fs::read_dir(&outputs[0])
            .expect("Failed to read output 0 dir")
            .count(),
        0
    );
    assert_ne!(
        fs::read_dir(&outputs[1])
            .expect("Failed to read output 1 dir")
            .count(),
        0
    );
    Ok(())
}
