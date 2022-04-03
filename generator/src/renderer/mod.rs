pub use renderer_config::RendererConfig;

mod case;
mod context;
mod primitive;
mod proto;
mod renderer_config;
pub mod scripted;
pub mod template;

pub const CONFIG_FILE_NAME: &'static str = "config.json";
