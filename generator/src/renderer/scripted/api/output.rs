use crate::renderer::renderer_config::{IndentChar, ScriptedConfig};
use unindent::unindent as unindent_multiline_str;

pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Output>()
        .register_fn("append", Output::append)
        .register_fn("line", Output::line)
        .register_fn("line", Output::newline)
        .register_fn("indent", Output::indent)
        .register_fn("unindent", Output::unindent)
        .register_fn("push_scope", Output::push_scope)
        .register_fn("pop_scope", Output::pop_scope);
}

/// NOTE: This API is used in rhai, so it follows rhai rules like always using &mut self and i64.
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
        if self.content.is_empty() || self.content.ends_with('\n') {
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

    fn push_scope(&mut self) {
        if self.config.scope.open_on_new_line {
            self.newline();
        } else if !self.content.ends_with(' ') && !self.content.ends_with('\n') {
            self.content.push(' ');
        }
        self.content.push_str(&self.config.scope.open);
        self.indent(self.config.scope.indent as i64);
        self.newline();
    }

    fn pop_scope(&mut self) {
        self.unindent(self.config.scope.indent as i64);
        if !self.content.ends_with('\n') {
            self.newline();
        }
        self.line("}");
    }

    pub fn to_string(self) -> String {
        self.content
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::renderer_config::{ScopeConfig, ScriptedConfig};
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

    mod push_scope {
        use crate::renderer::scripted::api::output::tests::scope_config;
        use crate::renderer::scripted::api::output::Output;

        #[test]
        fn adds_open_scope_str_with_space() {
            let mut output = Output::with_config(scope_config());
            output.append("0");
            output.push_scope();
            assert_eq!(&output.to_string(), "0 {\n");
        }

        #[test]
        fn does_not_add_space_if_already_exists() {
            let mut output = Output::with_config(scope_config());
            output.append("0 ");
            output.push_scope();
            assert_eq!(&output.to_string(), "0 {\n");
        }

        #[test]
        fn does_not_add_space_if_on_newline() {
            let mut output = Output::with_config(scope_config());
            output.line("0");
            output.push_scope();
            assert_eq!(&output.to_string(), "0\n{\n");
        }

        #[test]
        fn adds_open_scope_str_on_newline_if_configured() {
            let mut config = scope_config();
            config.scope.open_on_new_line = true;
            let mut output = Output::with_config(config);
            output.append("0");
            output.push_scope();
            assert_eq!(&output.to_string(), "0\n{\n");
        }

        #[test]
        fn adds_to_indent() {
            let mut output = Output::with_config(scope_config());
            output.append("0 ");
            output.push_scope();
            output.append("1");
            assert_eq!(&output.to_string(), "0 {\n  1");
        }
    }

    mod pop_scope {
        use crate::renderer::scripted::api::output::tests::scope_config;
        use crate::renderer::scripted::api::output::Output;

        #[test]
        fn adds_close_scope_str() {
            let mut output = Output::with_config(scope_config());
            output.pop_scope();
            assert_eq!(&output.to_string(), "\n}\n");
        }

        #[test]
        fn reduces_indent() {
            let mut output = Output::with_config(scope_config());
            output.append("0 ");
            output.push_scope();
            output.append("1");
            output.pop_scope();
            output.append("2");
            assert_eq!(&output.to_string(), "0 {\n  1\n}\n2");
        }

        #[test]
        fn only_adds_newline_if_not_already_on_newline() {
            let mut output = Output::with_config(scope_config());
            output.newline();
            output.pop_scope();
            assert_eq!(&output.to_string(), "\n}\n");
        }
    }

    fn scope_config() -> ScriptedConfig {
        ScriptedConfig {
            scope: ScopeConfig {
                open: "{".to_string(),
                close: "}".to_string(),
                indent: 2,
                open_on_new_line: false,
            },
            ..Default::default()
        }
    }
}
