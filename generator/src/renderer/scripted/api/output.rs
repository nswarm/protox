pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Output>()
        .register_fn("append", Output::append::<&str>)
        .register_fn("append", Output::append::<bool>)
        .register_fn("append", Output::append::<i64>)
        .register_fn("line", Output::line::<&str>)
        .register_fn("line", Output::line::<bool>)
        .register_fn("line", Output::line::<i64>);
}

#[derive(Default, Clone)]
pub struct Output {
    content: String,
}

impl Output {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<T: ToString>(&mut self, content: T) {
        self.content.push_str(&content.to_string());
    }

    pub fn line<T: ToString>(&mut self, content: T) {
        self.append(content);
        self.content.push('\n');
    }

    pub fn to_owned(self) -> String {
        self.content
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::scripted::api::output::Output;

    #[test]
    fn new_is_empty_content() {
        let output = Output::new();
        assert_eq!(output.to_owned(), "");
    }

    mod append {
        use crate::renderer::scripted::api::output::Output;

        #[test]
        fn appends_to_content() {
            let mut output = Output::new();
            output.append("000");
            output.append("111");
            output.append("222");
            assert_eq!(output.to_owned(), "000111222");
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
            assert_eq!(output.to_owned(), "000\n111\n222\n");
        }
    }
}
