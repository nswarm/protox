use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};
use anyhow::{anyhow, Result};

pub fn generate(request: &CodeGeneratorRequest) -> Result<CodeGeneratorResponse> {
    Ok(CodeGeneratorResponse {
        error: None,
        supported_features: None,
        file: vec![],
    })
}
