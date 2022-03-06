use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    Renderable,
};

#[derive(Clone, Copy)]
pub struct IfEquals;

impl HelperDef for IfEquals {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let lhs = h.param(0).ok_or(error_param_not_found("lhs"))?.value();
        let rhs = h.param(1).ok_or(error_param_not_found("rhs"))?.value();

        let template = if lhs == rhs {
            h.template()
        } else {
            h.inverse()
        };

        match template {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

fn error_param_not_found(name: &str) -> RenderError {
    RenderError::new(format!("Helper 'IfEquals': param '{}' not found", name))
}
