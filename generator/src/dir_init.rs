use crate::renderer::scripted::{MAIN_SCRIPT_NAME, SCRIPT_EXT};
use crate::renderer::template::{FILE_TEMPLATE_NAME, TEMPLATE_EXT};
use crate::renderer::RendererConfig;
use crate::{util, DEFAULT_CONFIG_FILE_NAME};
use anyhow::Result;
use std::io::Write;
use std::path::Path;
use unindent::unindent;

pub fn initialize_script_dir(dir: &Path) -> Result<()> {
    util::create_dir_or_error(dir)?;
    util::check_dir_is_empty(dir)?;
    write_config(dir)?;
    write_main_script(dir)?;
    Ok(())
}

pub fn initialize_template_dir(dir: &Path) -> Result<()> {
    util::create_dir_or_error(dir)?;
    util::check_dir_is_empty(dir)?;
    write_config(dir)?;
    write_file_template(dir)?;
    Ok(())
}

fn write_config(path: &Path) -> Result<()> {
    let config_file = util::create_file_or_error(&path.join(DEFAULT_CONFIG_FILE_NAME))?;
    let config = RendererConfig::default();
    serde_json::to_writer_pretty(config_file, &config)?;
    Ok(())
}

fn write_main_script(path: &Path) -> Result<()> {
    let mut file =
        util::create_file_or_error(&path.join(MAIN_SCRIPT_NAME).with_extension(SCRIPT_EXT))?;
    let contents = unindent(
        r#"
        // This is the root script file for most protox output.
        //
        // See the "builtin" and "examples" folders for usage examples:
        // https://github.com/nswarm/protox/tree/main/builtin
        // https://github.com/nswarm/protox/tree/main/examples
        //
        // See context data objects for information on what data is available:
        // https://github.com/nswarm/protox/tree/main/runner/src/renderer/context
        //
        // For more information on the scripting language rhai:
        // https://rhai.rs/book/

        // This function is the entrypoint that is called for each file protox expects to create.
        fn render_file(file, output) {
            // "file" is a FileContext from protox, usually related to a .proto file.
            // "output" is a protox object that makes it easy to construct an output file.

            // Add some stuff to the output...
            // Using "multiline" the leading indentation in code is ignored.
            output.multiline(`
                0
                    1
                        2
                    3
                4
            `);

            // Note the calling style of these two functions. The first has access to the parent scope,
            // the second does not.
            direct_fn!();
            output.line(indirect_fn(file.name));

            // Don't for get to return the output to protox!
            // The last line without a ; is returned.
            output
        }

        fn direct_fn() {
            // Add to existing output from parent scope.
            output.append(`Hello ${file.name}`);
        }

        // This function just returns a string which we can append outside.
        fn indirect_fn() {
            `Bye!`
        }
        "#,
    );
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn write_file_template(path: &Path) -> Result<()> {
    let mut file =
        util::create_file_or_error(&path.join(FILE_TEMPLATE_NAME).with_extension(TEMPLATE_EXT))?;
    let contents = unindent(
        r#"
        {{!
        This is the root template file for most protox output.

        See the "builtin" and "examples" folders for usage examples:
        https://github.com/nswarm/protox/tree/main/builtin
        https://github.com/nswarm/protox/tree/main/examples

        See context data objects for information on what data is available:
        https://github.com/nswarm/protox/tree/main/runner/src/renderer/context

        For more information on Handlebars: https://handlebarsjs.com/guide/
        }}"#,
    );
    file.write_all(contents.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::dir_init::initialize_template_dir;
    use crate::initialize_script_dir;
    use crate::renderer::scripted::{MAIN_SCRIPT_NAME, SCRIPT_EXT};
    use crate::renderer::template::{FILE_TEMPLATE_NAME, TEMPLATE_EXT};
    use crate::renderer::{RendererConfig, CONFIG_FILE_NAMES};
    use anyhow::Result;
    use std::fs;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn writes_config_file() -> Result<()> {
        let tempdir = tempdir()?;
        initialize_template_dir(tempdir.path())?;
        let config_file = fs::File::open(tempdir.path().join(CONFIG_FILE_NAMES[0]))?;
        let result: Result<RendererConfig, serde_json::Error> =
            serde_json::from_reader(config_file);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn writes_file_script_file() -> Result<()> {
        let tempdir = tempdir()?;
        initialize_script_dir(tempdir.path())?;
        let mut template_file = fs::File::open(
            tempdir
                .path()
                .join(MAIN_SCRIPT_NAME)
                .with_extension(SCRIPT_EXT),
        )?;
        let mut result = String::new();
        template_file.read_to_string(&mut result)?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[test]
    fn writes_file_template_file() -> Result<()> {
        let tempdir = tempdir()?;
        initialize_template_dir(tempdir.path())?;
        let mut template_file = fs::File::open(
            tempdir
                .path()
                .join(FILE_TEMPLATE_NAME)
                .with_extension(TEMPLATE_EXT),
        )?;
        let mut result = String::new();
        template_file.read_to_string(&mut result)?;
        assert!(!result.is_empty());
        Ok(())
    }
}
