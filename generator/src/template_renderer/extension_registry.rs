//! Create the ExtensionRegistry needed for idlx to function.
//! This is in a separate file so it's easy to find and register new options.
use prost::ExtensionRegistry;

pub fn create() -> ExtensionRegistry {
    let mut registry = ExtensionRegistry::new();
    registry.register(&proto_options::FILE_KEY_VALUE);
    registry.register(&proto_options::MSG_KEY_VALUE);
    registry.register(&proto_options::FIELD_KEY_VALUE);
    registry
}
