use anyhow::{anyhow, Result};

pub fn str_or_error<F: Fn() -> String>(value: &Option<String>, error: F) -> Result<&str> {
    let result = value
        .as_ref()
        .map(String::as_str)
        .ok_or(anyhow!("{}", error()))?;
    Ok(result)
}
