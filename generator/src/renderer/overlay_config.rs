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
    by_key: HashMap<Key, ValueTargets>,

    // Runtime cache.
    #[serde(skip)]
    target_to_kv: HashMap<Target, HashMap<Key, serde_yaml::Value>>,
}

impl OverlayConfig {
    #[cfg(test)]
    pub fn new(by_key: HashMap<Key, ValueTargets>) -> Self {
        let mut config = OverlayConfig {
            by_key,
            ..Default::default()
        };
        config.build_cache();
        config
    }

    pub fn get_all(&self, target: &str) -> Option<&HashMap<Key, serde_yaml::Value>> {
        self.target_to_kv.get(target)
    }

    pub fn build_cache(&mut self) {
        if self.is_cache_valid() {
            return;
        }
        for (key, vt) in &self.by_key {
            let value = &vt.value;
            let targets = &vt.targets;
            for target in targets {
                if !self.target_to_kv.contains_key(target) {
                    self.target_to_kv.insert(target.clone(), HashMap::new());
                }
                self.target_to_kv
                    .get_mut(target)
                    .unwrap()
                    .insert(key.clone(), value.clone());
            }
        }
    }

    fn is_cache_valid(&self) -> bool {
        if self.by_key.is_empty() {
            true
        } else {
            !self.target_to_kv.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::overlay_config::{Key, OverlayConfig, ValueTargets};
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;

    mod is_cache_valid {
        use crate::renderer::overlay_config::tests::test_by_key_data;
        use crate::renderer::overlay_config::OverlayConfig;

        #[test]
        fn always_true_without_by_key_data() {
            let mut config = OverlayConfig::default();
            assert!(config.is_cache_valid());
            config.build_cache();
            assert!(config.is_cache_valid());
        }

        #[test]
        fn true_after_build_cache_with_by_key_data() {
            let mut config = OverlayConfig {
                by_key: test_by_key_data(),
                target_to_kv: Default::default(),
            };
            assert!(!config.is_cache_valid());
            config.build_cache();
            assert!(config.is_cache_valid());
        }
    }

    #[test]
    fn new_builds_cache() {
        let config = OverlayConfig::new(test_by_key_data());
        assert!(config.is_cache_valid());
    }

    #[test]
    fn get_all() {
        let config = OverlayConfig::new(test_by_key_data());
        assert_eq!(
            config.get_all("target_0"),
            Some(&HashMap::from([
                ("test_key_0".to_string(), string_value("test_value_0")),
                ("test_key_1".to_string(), string_value("test_value_1")),
            ]))
        );
    }

    fn test_by_key_data() -> HashMap<Key, ValueTargets> {
        HashMap::from([
            (
                "test_key_0".to_string(),
                ValueTargets::new("test_value_0", &["target_0"]),
            ),
            (
                "test_key_1".to_string(),
                ValueTargets::new("test_value_1", &["target_0", "target_1"]),
            ),
        ])
    }

    fn string_value(value: &str) -> serde_yaml::Value {
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
