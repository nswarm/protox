use std::collections::HashMap;

pub trait Overlayed {
    fn overlays(&self) -> &HashMap<String, serde_yaml::Value>;

    fn overlay(&self, key: &str) -> serde_yaml::Value {
        self.overlays()
            .get(key)
            .map(|x| x.clone())
            .unwrap_or(serde_yaml::Value::Null)
    }
}
