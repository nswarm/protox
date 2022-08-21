use anyhow::Result;
use prost_types::FileDescriptorSet;
use std::path::{Path, PathBuf};

pub trait Render {
    /// Load any necessary files from the `input_root` directory and overlays as specified.
    fn load(&mut self, input_root: &Path, overlays: &[PathBuf]) -> Result<()>;
    /// Reset is called between runs with different input/outputs.
    fn reset(&mut self);
    /// Do the actual rendering to the `output_path` directory.
    fn render(&self, descriptor_set: &FileDescriptorSet, output_path: &Path) -> Result<()>;
}
