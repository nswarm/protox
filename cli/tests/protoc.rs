use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir_in, TempDir};

macro_rules! protoc_test {
    ($fn_name: ident, $test_name: ident, $lang: expr) => {
        #[test]
        fn $test_name() -> Result<()> {
            $fn_name($lang)?;
            Ok(())
        }
    };
}

mod test_lang {
    use crate::test_with_args;
    use anyhow::Result;
    use runner::Lang;
    use std::fs;

    protoc_test!(test_lang, cpp, Lang::Cpp);
    protoc_test!(test_lang, csharp, Lang::CSharp);
    protoc_test!(test_lang, java, Lang::Java);
    protoc_test!(test_lang, js, Lang::Javascript);
    protoc_test!(test_lang, kotlin, Lang::Kotlin);
    protoc_test!(test_lang, objc, Lang::ObjectiveC);
    protoc_test!(test_lang, php, Lang::Php);
    protoc_test!(test_lang, python, Lang::Python);
    protoc_test!(test_lang, ruby, Lang::Ruby);
    protoc_test!(test_lang, rust, Lang::Rust);

    fn test_lang(lang: Lang) -> Result<()> {
        let output_dir = test_with_args(&["--proto", &lang.as_config()])?;
        assert_ne!(fs::read_dir(output_dir.path())?.count(), 0);
        Ok(())
    }
}

mod test_protoc_extra_args {
    use crate::test_with_args_in;
    use anyhow::Result;
    use runner::Lang;
    use std::fs;
    use tempfile::tempdir_in;

    protoc_test!(test_protoc_extra_args, cpp, Lang::Cpp);
    protoc_test!(test_protoc_extra_args, csharp, Lang::CSharp);
    protoc_test!(test_protoc_extra_args, java, Lang::Java);
    protoc_test!(test_protoc_extra_args, js, Lang::Javascript);
    protoc_test!(test_protoc_extra_args, kotlin, Lang::Kotlin);
    protoc_test!(test_protoc_extra_args, objc, Lang::ObjectiveC);
    protoc_test!(test_protoc_extra_args, php, Lang::Php);
    protoc_test!(test_protoc_extra_args, python, Lang::Python);
    protoc_test!(test_protoc_extra_args, ruby, Lang::Ruby);
    protoc_test!(test_protoc_extra_args, rust, Lang::Rust);

    fn test_protoc_extra_args(lang: Lang) -> Result<()> {
        let output_dir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
        let expected_filename = "expected_file";
        let expected_file_path = output_dir
            .path()
            .join(expected_filename)
            .to_str()
            .unwrap()
            .to_string();
        test_with_args_in(
            &output_dir,
            &[
                "--proto",
                &lang.as_config(),
                "--protoc-args",
                &format!("\"--dependency_out={}\"", expected_file_path),
            ],
        )?;
        let mut found_expected_file = false;
        for entry in fs::read_dir(output_dir.path())? {
            if entry?.file_name() == expected_filename {
                found_expected_file = true;
                break;
            }
        }
        assert!(
            found_expected_file,
            "--protoc-args were not correctly passed to protoc."
        );
        Ok(())
    }
}

fn test_with_args(args: &[&str]) -> Result<TempDir> {
    let tempdir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    test_with_args_in(&tempdir, args)?;
    Ok(tempdir)
}

fn test_with_args_in(dir: &TempDir, args: &[&str]) -> Result<()> {
    let mut cmd = struct_ffi_gen();
    cmd.env("RUST_LOG", "debug")
        .arg("--input")
        .arg(path_to_str(resources_dir())?)
        .arg("--output-root")
        .arg(path_to_str(dir.path())?);

    for arg in args {
        cmd.arg(arg);
    }

    assert_cmd(cmd)?;
    Ok(())
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
