use rhai;
use rhai::exported_module;
use rhai::plugin::*;

use crate::renderer::scripted::api::output::Output;

pub mod output;

pub fn register(engine: &mut rhai::Engine) {
    output(engine);
    context(engine);
}

fn output(engine: &mut rhai::Engine) {
    engine
        .register_type::<Output>()
        .register_fn("append", Output::append);
}

fn context(engine: &mut rhai::Engine) {
    engine.register_global_module(exported_module!(context).into());
}

#[export_module]
mod context {
    use crate::renderer::context;

    pub type FileContext = context::FileContext;
    pub type EnumContext = context::EnumContext;
    pub type MessageContext = context::MessageContext;
    pub type FieldContext = context::FieldContext;
    pub type MetadataContext = context::MetadataContext;
    pub type ImportContext = context::ImportContext;

    ////////////////////////////////////////////////////
    // FileContext

    #[rhai_fn(get = "source_file")]
    pub fn source_name(context: &mut FileContext) -> String {
        context.source_file().to_string()
    }
    #[rhai_fn(get = "imports")]
    pub fn imports(context: &mut FileContext) -> rhai::Dynamic {
        context.imports().clone().into()
    }
    #[rhai_fn(get = "enums")]
    pub fn enums(context: &mut FileContext) -> rhai::Dynamic {
        context.enums().clone().into()
    }
    #[rhai_fn(get = "messages")]
    pub fn messages(context: &mut FileContext) -> rhai::Dynamic {
        context.messages().clone().into()
    }
    // #[rhai_fn(get = "options")]
    // pub fn options(context: &mut FileContext) -> Option<FileOptions> {
    //     context.options().clone()
    // }

    ////////////////////////////////////////////////////
    // MessageContext

    #[rhai_fn(get = "name")]
    pub fn name(context: &mut MessageContext) -> String {
        context.name().to_string()
    }

    mod field {}

    mod metadata {}

    mod import {}
}
