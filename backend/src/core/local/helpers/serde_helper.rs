use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

/// Deserialize FlatBuffer's [{key, value}] array format into HashMap
pub fn deserialize_fb_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
{
    #[derive(Deserialize)]
    struct KV<K, V> {
        key: K,
        value: V,
    }

    let items: Vec<KV<K, V>> = Vec::deserialize(deserializer)?;
    Ok(items.into_iter().map(|kv| (kv.key, kv.value)).collect())
}
