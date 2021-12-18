mod generator;

use prost::Message;
use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};
use std::io::{BufRead, Write};
use std::{io, process};
use anyhow::{Error, Result};

const ERROR_FAILED_TO_WRITE_RESPONSE: i32 = 100;

fn main() {
    let response = match parse_request_from_stdin() {
        Err(err) => generate_error(&err),
        Ok(request) => generate(&request),
    };

    if let Err(_) = write_response_to_stdout(&response) {
        process::exit(ERROR_FAILED_TO_WRITE_RESPONSE);
    }
}

fn parse_request_from_stdin() -> Result<CodeGeneratorRequest> {
    let stdin = io::stdin();
    let mut lock = stdin.lock();
    let buf = lock.fill_buf()?;
    let request = CodeGeneratorRequest::decode(buf)?;
    Ok(request)
}

fn write_response_to_stdout(response: &CodeGeneratorResponse) -> Result<()> {
    let mut bytes = Vec::new();
    response.encode(&mut bytes)?;

    let mut stdout = io::stdout();
    stdout.lock().write_all(&bytes)?;
    stdout.flush()?;
    Ok(())
}

fn generate_error(err: &Error) -> CodeGeneratorResponse {
    CodeGeneratorResponse {
        error: Some(format!("ERROR\n{:?}", err)),
        supported_features: None,
        file: vec![],
    }
}

fn generate(request: &CodeGeneratorRequest) -> CodeGeneratorResponse {
    match generator::generate(request) {
        Ok(response) => response,
        Err(err) => generate_error(&err),
    }
}
