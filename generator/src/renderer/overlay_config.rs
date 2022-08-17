use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type Target = String;
pub type Key = String;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ValueTargets {
    pub value: serde_yaml::Value,
    pub targets: HashSet<Target>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OverlayConfig {
    // Config only, not complete after initialization.
    #[serde(default)]
    by_key: HashMap<Key, ValueTargets>,

    // Modified during initialization to include all data from by_key.
    #[serde(default)]
    by_target: HashMap<Target, HashMap<Key, serde_yaml::Value>>,

    #[serde(skip)]
    is_initialized: bool,
}

impl OverlayConfig {
    #[cfg(test)]
    pub fn new(
        by_key: HashMap<Key, ValueTargets>,
        by_target: HashMap<Target, HashMap<Key, serde_yaml::Value>>,
    ) -> Self {
        let mut config = OverlayConfig {
            by_key,
            by_target,
            is_initialized: false,
        };
        config.initialize();
        config
    }

    pub fn by_target(&self, target: &str) -> Option<&HashMap<Key, serde_yaml::Value>> {
        self.by_target.get(target)
    }

    pub fn by_target_opt_clone(
        &self,
        target: &Option<String>,
    ) -> HashMap<String, serde_yaml::Value> {
        if let Some(name) = target {
            self.by_target(name)
                .map(Clone::clone)
                .unwrap_or(HashMap::new())
        } else {
            HashMap::new()
        }
    }

    pub fn initialize(&mut self) {
        if self.is_initialized {
            return;
        }
        self.is_initialized = true;
        for (key, vt) in &self.by_key {
            let value = &vt.value;
            let targets = &vt.targets;
            for target in targets {
                if !self.by_target.contains_key(target) {
                    self.by_target.insert(target.clone(), HashMap::new());
                }
                let kv = self.by_target.get_mut(target).unwrap();
                // Don't overwrite!
                if !kv.contains_key(key) {
                    kv.insert(key.clone(), value.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::overlay_config::{Key, OverlayConfig, Target, ValueTargets};
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;

    macro_rules! by_key {
        ( $($e:expr),* ) => {
            HashMap::from([ $($e,)* ])
        };
    }

    macro_rules! by_target {
        ( $($e:expr),* ) => {
            HashMap::from([ $($e,)* ])
        };
    }

    mod is_initialized {
        use crate::renderer::overlay_config::tests::{arbitrary_by_key, arbitrary_by_target};
        use crate::renderer::overlay_config::OverlayConfig;

        #[test]
        fn true_after_initialize_with_by_key_config() {
            let mut config = OverlayConfig {
                by_key: arbitrary_by_key(),
                ..Default::default()
            };
            assert!(!config.is_initialized);
            config.initialize();
            assert!(config.is_initialized);
        }

        #[test]
        fn true_after_initialize_with_by_target_config() {
            let mut config = OverlayConfig {
                by_target: arbitrary_by_target(),
                ..Default::default()
            };
            assert!(!config.is_initialized);
            config.initialize();
            assert!(config.is_initialized);
        }
    }

    #[test]
    fn new_initializes_automatically() {
        let config = OverlayConfig::new(arbitrary_by_key(), arbitrary_by_target());
        assert!(config.is_initialized);
    }

    mod get_by_target {
        use crate::renderer::overlay_config::tests::{by_key_entry, by_target_entry, yaml_string};
        use crate::renderer::overlay_config::OverlayConfig;
        use std::collections::HashMap;

        #[test]
        fn from_by_key_data() {
            let config = OverlayConfig::new(
                by_key!(
                    by_key_entry("key0", "value0", &["target0"]),
                    by_key_entry("key1", "value1", &["target0", "target1"])
                ),
                by_target!(),
            );
            assert_eq!(
                config.by_target("target0"),
                Some(&HashMap::from([
                    ("key0".to_string(), yaml_string("value0")),
                    ("key1".to_string(), yaml_string("value1")),
                ]))
            );
            assert_eq!(
                config.by_target("target1"),
                Some(&HashMap::from([(
                    "key1".to_string(),
                    yaml_string("value1")
                )]))
            );
        }

        #[test]
        fn from_by_target_data() {
            let config = OverlayConfig::new(
                by_key!(),
                by_target!(
                    by_target_entry("target0", &[("key0", "value0"), ("key1", "value1")]),
                    by_target_entry("target1", &[("key0", "value1"), ("key2", "value2")])
                ),
            );
            assert_eq!(
                config.by_target("target0"),
                Some(&HashMap::from([
                    ("key0".to_string(), yaml_string("value0")),
                    ("key1".to_string(), yaml_string("value1")),
                ]))
            );
            assert_eq!(
                config.by_target("target1"),
                Some(&HashMap::from([
                    // key0 with different value than target0.
                    ("key0".to_string(), yaml_string("value1")),
                    ("key2".to_string(), yaml_string("value2")),
                ]))
            );
        }

        #[test]
        fn both_by_key_and_target() {
            let config = OverlayConfig::new(
                by_key!(by_key_entry("key0", "value0", &["target0"])),
                by_target!(by_target_entry("target0", &[("key1", "value1")])),
            );
            assert_eq!(
                config.by_target("target0"),
                Some(&HashMap::from([
                    ("key0".to_string(), yaml_string("value0")),
                    ("key1".to_string(), yaml_string("value1")),
                ]))
            );
        }

        #[test]
        fn by_target_overrides_by_key() {
            let config = OverlayConfig::new(
                by_key!(by_key_entry("key0", "value0", &["target0"])),
                by_target!(by_target_entry("target0", &[("key0", "override_value!")])),
            );
            assert_eq!(
                config.by_target("target0"),
                Some(&HashMap::from([(
                    "key0".to_string(),
                    yaml_string("override_value!")
                ),]))
            );
        }
    }

    fn arbitrary_by_key() -> HashMap<Key, ValueTargets> {
        by_key!(
            by_key_entry("key0", "value0", &["target0"]),
            by_key_entry("key0", "value0", &["target0", "target1"])
        )
    }

    fn arbitrary_by_target() -> HashMap<Target, HashMap<Key, serde_yaml::Value>> {
        by_target!(
            by_target_entry("target0", &[("key2", "value2"), ("key3", "value3")]),
            by_target_entry("target1", &[("key0", "value5")])
        )
    }

    fn by_key_entry(key: &str, value: &str, targets: &[&str]) -> (String, ValueTargets) {
        (key.to_string(), ValueTargets::new(value, targets))
    }

    fn by_target_entry(
        target: &str,
        kv: &[(&str, &str)],
    ) -> (String, HashMap<Key, serde_yaml::Value>) {
        (
            target.to_string(),
            HashMap::from_iter(kv.into_iter().map(|(k, v)| (k.to_string(), yaml_string(v)))),
        )
    }

    fn yaml_string(value: &str) -> serde_yaml::Value {
        serde_yaml::Value::String(value.to_owned())
    }

    impl ValueTargets {
        pub fn new(value: &str, targets: &[&str]) -> Self {
            Self {
                value: serde_yaml::Value::String(value.to_owned()),
                targets: HashSet::from_iter(targets.into_iter().map(|x| x.to_string())),
            }
        }
    }
}
