mod util;

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
    use crate::util;
    use anyhow::{Context, Result};
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
        let lang_name = lang.as_config();
        let output_dir = util::test_with_args(&["--proto", &lang_name, &lang_name])?;
        let lang_output_dir = output_dir.path().join(&lang_name);
        let output_files = fs::read_dir(&lang_output_dir)
            .context("Missing output dir for lang")?
            .count();
        assert_ne!(output_files, 0);
        Ok(())
    }
}

mod test_protoc_extra_args {
    use crate::util;
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
        util::test_with_args_in(
            output_dir.path(),
            &[
                "--proto",
                &lang.as_config(), // INPUT
                &lang.as_config(), // OUTPUT
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
