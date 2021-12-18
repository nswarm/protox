use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};
use anyhow::{Result};

pub fn generate(_request: &CodeGeneratorRequest) -> Result<CodeGeneratorResponse> {
    Ok(CodeGeneratorResponse {
        error: None,
        supported_features: None,
        file: vec![],
    })
}
