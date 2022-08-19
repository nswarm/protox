use prost::ExtensionRegistry;
use rhai;
// use rhai::exported_module;

pub use extensions::*;

pub(crate) mod extensions {
    include!(concat!(env!("OUT_DIR"), "/protox.rs"));

    // Example:
    //
    // pub mod my_mod {
    //     include!(concat!(env!("OUT_DIR"), "/my_mod.rs"));
    // }

    // --- user generated file includes ---
    // ------------------------------------
}

pub fn create_extension_registry() -> ExtensionRegistry {
    let mut registry = ExtensionRegistry::new();
    register_builtin_extensions(&mut registry);
    register_user_extensions(&mut registry);
    registry
}

pub fn register_script_apis(engine: &mut rhai::Engine) {
    // Note: protox script API lives in the generator project.
    register_user_script_apis(engine);
}

fn register_builtin_extensions(registry: &mut ExtensionRegistry) {
    registry.register(extensions::NATIVE_TYPE);
}

#[allow(unused)]
fn register_user_extensions(registry: &mut ExtensionRegistry) {}

#[allow(unused)]
fn register_user_script_apis(engine: &mut rhai::Engine) {
    // engine.register_global_module(exported_module!(<<namespace::api>>).into());
}
