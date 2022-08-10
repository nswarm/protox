use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

type Target = String;
type Key = String;

#[derive(Serialize, Deserialize, Clone, Default)]
struct ValueTargets {
    value: serde_yaml::Value,
    targets: HashSet<Target>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OverlayConfig {
    by_key: HashMap<Key, ValueTargets>,

    // Runtime cache.
    #[serde(skip)]
    target_to_kv: HashMap<Target, HashMap<Key, serde_yaml::Value>>,
}

impl OverlayConfig {
    pub fn get_all(&self, target: &str) -> Option<&HashMap<Key, serde_yaml::Value>> {
        self.target_to_kv.get(target)
    }

    pub fn build_cache(&mut self) {
        if !self.needs_cache_build() {
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

    fn needs_cache_build(&self) -> bool {
        self.target_to_kv.is_empty() && !self.by_key.is_empty()
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn asdf() {
//         todo!("add tests!")
//     }
// }
