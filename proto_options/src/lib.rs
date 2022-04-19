mod extensions {
    include!(concat!(env!("OUT_DIR"), "/protox.rs"));
}

pub use extensions::*;

use prost::ExtensionRegistry;

pub fn create_extension_registry() -> ExtensionRegistry {
    let mut registry = ExtensionRegistry::new();
    // protox built-in
    registry.register(extensions::FILE_KEY_VALUE);
    registry.register(extensions::ENUM_KEY_VALUE);
    registry.register(extensions::MSG_KEY_VALUE);
    registry.register(extensions::FIELD_KEY_VALUE);
    registry.register(extensions::NATIVE_TYPE);

    // --- space for user additions ---
    // --------------------------------
    registry
}
