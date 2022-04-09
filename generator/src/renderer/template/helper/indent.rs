use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    Renderable, StringOutput,
};

#[derive(Clone, Copy)]
pub struct Indent;

impl HelperDef for Indent {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"indent\""))?;
        let num_spaces = param.value().as_i64().unwrap_or(0) as usize;

        let template = match h.template() {
            Some(t) => t,
            None => return Ok(()),
        };

        let mut output = StringOutput::new();
        template.render(r, ctx, rc, &mut output)?;
        let rendered = indent(&output.into_string()?, num_spaces);

        out.write(&rendered)?;

        Ok(())
    }
}

fn indent(content: &str, num_spaces: usize) -> String {
    let whitespace = vec![" "; num_spaces].concat();
    content
        .split('\n')
        .map(|s| {
            if !s.trim().is_empty() {
                [&whitespace, s].concat()
            } else {
                s.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::renderer::template::helper::indent::indent;

    #[test]
    fn test_indent() {
        let content = r#"
0
    1
        2
    3
4
"#;

        assert_eq!(
            indent(content, 2),
            r#"
  0
      1
          2
      3
  4
"#
        );

        assert_eq!(
            indent(content, 4),
            r#"
    0
        1
            2
        3
    4
"#
        );
    }
}
