use anyhow::{anyhow, Result};
use runner::Lang;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir_in, TempDir};

macro_rules! test_lang {
    ($name: ident, $lang: expr) => {
        #[test]
        fn $name() -> Result<()> {
            test_lang($lang)?;
            Ok(())
        }
    };
}

test_lang!(cpp, Lang::Cpp);
test_lang!(csharp, Lang::CSharp);
test_lang!(java, Lang::Java);
test_lang!(js, Lang::Javascript);
test_lang!(kotlin, Lang::Kotlin);
test_lang!(objc, Lang::ObjectiveC);
test_lang!(php, Lang::Php);
test_lang!(python, Lang::Python);
test_lang!(ruby, Lang::Ruby);
test_lang!(rust, Lang::Rust);

fn test_lang(lang: Lang) -> Result<()> {
    let output_dir = test_with_args(&["--proto", &lang.as_config()])?;
    assert_ne!(fs::read_dir(output_dir.path())?.count(), 0);
    Ok(())
}

fn test_with_args(args: &[&str]) -> Result<TempDir> {
    let tempdir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    let mut cmd = struct_ffi_gen();
    cmd.env("RUST_LOG", "debug")
        .arg("--input")
        .arg(path_to_str(resources_dir())?)
        .arg("--output-root")
        .arg(path_to_str(tempdir.path())?);

    for arg in args {
        cmd.arg(arg);
    }

    assert_cmd(cmd)?;
    Ok(tempdir)
}

fn assert_cmd(mut cmd: Command) -> Result<()> {
    println!("Full command: {:?}\n", cmd);
    let output = cmd.output()?;
    println!(
        "::: stdout :::\n{}\n::: stderr :::\n{}",
        String::from_utf8(output.stdout.clone())?,
        String::from_utf8(output.stderr.clone())?,
    );
    assert!(output.status.success());
    Ok(())
}

fn resources_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources");
    path
}

fn path_to_str<P: AsRef<Path>>(path: P) -> Result<String> {
    path.as_ref()
        .to_str()
        .map(&str::to_string)
        .ok_or(anyhow!("Invalid path: {:?}", path.as_ref()))
}

fn struct_ffi_gen() -> Command {
    Command::new(env!("CARGO_BIN_EXE_struct-ffi-gen"))
}
