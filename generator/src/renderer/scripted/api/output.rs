use unindent::unindent;

pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Output>()
        .register_fn("append", Output::append)
        .register_fn("line", Output::line);
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
        // trim_start + unindent so it's always aligned as expected.
        self.content.push_str(unindent(&content).trim_start());
    }

    pub fn line(&mut self, content: &str) {
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

    #[test]
    fn append_adds_to_content_without_newline() {
        let mut output = Output::new();
        output.append("000");
        output.append("111");
        output.append("222");
        assert_eq!(output.to_owned(), "000111222");
    }

    #[test]
    fn line_adds_to_content_with_newline() {
        let mut output = Output::new();
        output.line("000");
        output.line("111");
        output.line("222");
        assert_eq!(output.to_owned(), "000\n111\n222\n");
    }
}
