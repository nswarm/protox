use prost::ExtensionRegistry;
use rhai;
use rhai::exported_module;

pub use extensions::*;

pub(crate) mod extensions {
    include!(concat!(env!("OUT_DIR"), "/protox.rs"));
    pub mod fbs {
        include!(concat!(env!("OUT_DIR"), "/fbs.rs"));
    }

    // --- user generated file includes ---
    // ------------------------------------
}

mod fbs;

pub fn create_extension_registry() -> ExtensionRegistry {
    let mut registry = ExtensionRegistry::new();
    register_builtin_extensions(&mut registry);
    register_user_extensions(&mut registry);
    registry
}

pub fn register_script_apis(engine: &mut rhai::Engine) {
    // Note: protox script API lives in the generator project.
    engine.register_global_module(exported_module!(fbs::api).into());
    register_user_script_apis(engine);
}

fn register_builtin_extensions(registry: &mut ExtensionRegistry) {
    // Key-value options.
    registry.register(extensions::FILE_KEY_VALUE);
    registry.register(extensions::ENUM_KEY_VALUE);
    registry.register(extensions::MSG_KEY_VALUE);
    registry.register(extensions::FIELD_KEY_VALUE);
    registry.register(extensions::NATIVE_TYPE);
    // Fbs options.
    registry.register(extensions::fbs::ENUM_TYPE);
    registry.register(extensions::fbs::MESSAGE_TYPE);
    registry.register(extensions::fbs::FIELD_TYPE);
}

#[allow(unused)]
fn register_user_extensions(registry: &mut ExtensionRegistry) {}

#[allow(unused)]
fn register_user_script_apis(engine: &mut rhai::Engine) {}
