//! Handling related to arbitrary key=value pairs specified in custom idlx proto options.
use anyhow::{anyhow, Result};
use prost::extension::ExtensionSetError;
use prost::{Extendable, Extension, ExtensionImpl};
use std::collections::HashMap;

const KV_SEPARATOR: &str = "=";

pub fn insert_custom_options<E: Extendable>(
    map: &mut HashMap<String, serde_json::Value>,
    options: &E,
    extension: &ExtensionImpl<Vec<String>>,
) -> Result<()> {
    match options.extension_data(extension) {
        Err(err) => {
            extension_set_error(err, E::extendable_type_id(), extension.extendable_type_id())
        }
        Ok(value) => {
            for kv in value {
                let (key, value) = split_kv_or_error(kv)?;
                map.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
            }
            Ok(())
        }
    }
}

fn extension_set_error(
    err: ExtensionSetError,
    target_type_id: &str,
    extension_type_id: &str,
) -> Result<()> {
    match err {
        ExtensionSetError::ExtensionNotFound => Ok(()),
        ExtensionSetError::WrongExtendableTypeId => Err(anyhow!(
            "Using wrong Extension with Extendable. Target: {}, expected {}",
            target_type_id,
            extension_type_id,
        )),
        ExtensionSetError::CastFailed => {
            Err(anyhow!("Failed to cast custom option data to Vec<String>"))
        }
    }
}

fn split_kv_or_error(kv: &str) -> Result<(&str, &str)> {
    let error_msg = || {
        anyhow!(
            "Failed to split custom key-value option '{}'. Expected format: key=value",
            kv
        )
    };
    let (k, v) = kv.split_once(KV_SEPARATOR).ok_or(error_msg())?;
    if k.is_empty() || v.is_empty() {
        return Err(error_msg());
    }
    Ok((k, v))
}

#[cfg(test)]
mod tests {
    use crate::renderer::context::option_key_value::insert_custom_options;
    use anyhow::Result;
    use prost::Extendable;
    use prost_types::FileOptions;
    use std::collections::HashMap;

    #[test]
    fn test_insert_custom_options() -> Result<()> {
        let mut map = HashMap::<String, serde_json::Value>::new();
        let mut options = FileOptions::default();
        let extension = &proto_options::FILE_KEY_VALUE;
        options.set_extension_data(
            extension,
            vec!["key0=value0".to_owned(), "key1=value1".to_owned()],
        )?;
        insert_custom_options(&mut map, &options, extension)?;
        assert_eq!(
            map.get("key0"),
            Some(&serde_json::Value::String("value0".to_owned()))
        );
        assert_eq!(
            map.get("key1"),
            Some(&serde_json::Value::String("value1".to_owned()))
        );
        Ok(())
    }

    mod split_kv_or_error {
        use crate::renderer::context::option_key_value::split_kv_or_error;
        use anyhow::Result;

        #[test]
        fn single_separator() -> Result<()> {
            assert_eq!(split_kv_or_error("key=value")?, ("key", "value"));
            Ok(())
        }

        #[test]
        fn multiple_separators() -> Result<()> {
            assert_eq!(split_kv_or_error("key==value")?, ("key", "=value"));
            assert_eq!(split_kv_or_error("key=value=asd")?, ("key", "value=asd"));
            Ok(())
        }

        #[test]
        fn empty_key() {
            assert!(split_kv_or_error("=value").is_err());
        }

        #[test]
        fn empty_value() {
            assert!(split_kv_or_error("key=").is_err());
        }

        #[test]
        fn no_separator() {
            assert!(split_kv_or_error("keyvalue").is_err());
        }

        #[test]
        fn invalid_separator() {
            assert!(split_kv_or_error("key;value").is_err());
        }
    }
}
