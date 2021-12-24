#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use anyhow::{anyhow, Result};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::{tempdir_in, TempDir};

    pub fn test_with_args(args: &[&str]) -> Result<TempDir> {
        let tempdir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
        test_with_args_in(tempdir.path(), args)?;
        Ok(tempdir)
    }

    pub fn test_with_args_in<A: AsRef<str>>(output: &Path, args: &[A]) -> Result<()> {
        let mut cmd = protoffi();
        cmd.env("RUST_LOG", "debug,handlebars=off")
            .arg("--input")
            .arg(path_to_str(resources_dir())?)
            .arg("--output-root")
            .arg(path_to_str(output)?);

        for arg in args {
            cmd.arg(arg.as_ref());
        }

        assert_cmd(cmd)?;
        Ok(())
    }

    pub fn assert_cmd(mut cmd: Command) -> Result<()> {
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

    pub fn resources_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources");
        path
    }

    pub fn path_to_str<P: AsRef<Path>>(path: P) -> Result<String> {
        path.as_ref()
            .to_str()
            .map(&str::to_string)
            .ok_or(anyhow!("Invalid path: {:?}", path.as_ref()))
    }

    pub fn protoffi() -> Command {
        Command::new(env!("CARGO_BIN_EXE_protoffi"))
    }
}

pub use tests::*;
