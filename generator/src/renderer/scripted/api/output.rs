use crate::renderer::renderer_config::{IndentChar, ScriptedConfig};
use unindent::unindent as unindent_multiline_str;

pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Output>()
        .register_fn("append", Output::append)
        .register_fn("line", Output::line)
        .register_fn("line", Output::newline)
        .register_fn("indent", Output::indent)
        .register_fn("unindent", Output::unindent);
}

#[derive(Default, Clone)]
pub struct Output {
    config: ScriptedConfig,
    content: String,
    current_indent: i64,
}

impl Output {
    pub fn with_config(config: ScriptedConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn append(&mut self, new_content: &str) {
        if self.content.is_empty() || self.content.ends_with("\n") {
            self.push_indent();
        }
        // trim_start + unindent lets users use multiline strings indented inside of their
        // code and it will won't have the indent in output.
        self.content
            .push_str(unindent_multiline_str(new_content).trim_start());
    }

    pub fn line(&mut self, content: &str) {
        self.append(content);
        self.newline();
    }

    pub fn newline(&mut self) {
        self.content.push('\n');
    }

    pub fn indent(&mut self, amount: i64) {
        self.current_indent += amount;
    }

    pub fn unindent(&mut self, amount: i64) {
        self.current_indent = std::cmp::max(self.current_indent - amount, 0);
    }

    pub fn indent_char(&self) -> char {
        match self.config.indent_char {
            IndentChar::Space => ' ',
            IndentChar::Tab => '\t',
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.current_indent {
            self.content.push(self.indent_char())
        }
    }

    pub fn to_string(self) -> String {
        self.content
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::scripted::api::output::Output;

    #[test]
    fn new_is_empty_content() {
        let output = Output::default();
        assert_eq!(output.to_string(), "");
    }

    #[test]
    fn append_adds_to_content_without_newline() {
        let mut output = Output::default();
        output.append("000");
        output.append("111");
        output.append("222");
        assert_eq!(output.to_string(), "000111222");
    }

    #[test]
    fn line_adds_to_content_with_newline() {
        let mut output = Output::default();
        output.line("000");
        output.line("111");
        output.line("222");
        assert_eq!(output.to_string(), "000\n111\n222\n");
    }

    mod indent {
        use crate::renderer::scripted::api::output::Output;

        mod char {
            use crate::renderer::renderer_config::{IndentChar, ScriptedConfig};
            use crate::renderer::scripted::api::output::Output;

            #[test]
            fn space() {
                run_test(IndentChar::Space, 2, "  x");
            }

            #[test]
            fn tab() {
                run_test(IndentChar::Tab, 2, "\t\tx");
            }

            fn run_test(indent_char: IndentChar, amount: i64, expected: &str) {
                let config = ScriptedConfig {
                    indent_char,
                    ..Default::default()
                };
                let mut output = Output::with_config(config);
                output.indent(amount);
                output.append("x");
                assert_eq!(&output.to_string(), expected);
            }
        }

        #[test]
        fn applies_immediately_if_start_of_output() {
            let mut output = Output::default();
            output.indent(2);
            output.append("0");
            assert_eq!(&output.to_string(), "  0");
        }

        #[test]
        fn applies_immediately_if_directly_after_newline() {
            let mut output = Output::default();
            output.line("0");
            output.indent(2);
            output.append("1");
            assert_eq!(&output.to_string(), "0\n  1");
        }

        #[test]
        fn applies_after_next_newline_if_not_after_newline() {
            let mut output = Output::default();
            output.append("0");
            output.indent(2);
            output.line("1");
            output.append("2");
            assert_eq!(&output.to_string(), "01\n  2");
        }

        #[test]
        fn applies_to_subsequent_lines() {
            let mut output = Output::default();
            output.line("0");
            output.indent(1);
            output.line("1");
            output.line("2");
            output.line("3");
            assert_eq!(&output.to_string(), "0\n 1\n 2\n 3\n");
        }

        #[test]
        fn unindent_reduces_indent_for_subsequent_lines() {
            let mut output = Output::default();
            output.line("0");
            output.indent(2);
            output.line("1");
            output.unindent(1);
            output.line("2");
            output.line("3");
            assert_eq!(&output.to_string(), "0\n  1\n 2\n 3\n");
        }

        #[test]
        fn indents_are_cumulative() {
            let mut output = Output::default();
            output.line("0");
            output.indent(1);
            output.line("1");
            output.indent(1);
            output.line("2");
            output.indent(2);
            output.line("3");
            assert_eq!(&output.to_string(), "0\n 1\n  2\n    3\n");
        }
    }
}
