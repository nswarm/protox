use rhai::plugin::*;

pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Output>()
        .register_fn("append", Output::append)
        .register_fn("append_inline", Output::append);
}

#[derive(Default, Clone)]
pub struct Output {
    content: String,
}

impl Output {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, content: &str) {
        self.content.push_str(content);
    }

    pub fn line(&mut self, content: &str) {
        self.append(content);
        self.content.push('\n');
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
        let output = Output::new();
        assert_eq!(output.to_string(), "");
    }

    mod append {
        use crate::renderer::scripted::api::output::Output;

        #[test]
        fn appends_to_content() {
            let mut output = Output::new();
            output.append("000");
            output.append("111");
            output.append("222");
            assert_eq!(output.to_string(), "000111222");
        }
    }

    mod line {
        use crate::renderer::scripted::api::output::Output;

        #[test]
        fn appends_to_content_with_newline() {
            let mut output = Output::new();
            output.line("000");
            output.line("111");
            output.line("222");
            assert_eq!(output.to_string(), "000\n111\n222\n");
        }
    }
}
