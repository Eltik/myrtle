//! Serde helpers for deserializing FlatBuffer JSON format.
//!
//! FlatBuffer outputs dictionaries as arrays of key-value pairs:
//! `[{ "key": "id", "value": {...} }]`
//!
//! These helpers convert that format to standard HashMaps.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Key-value pair structure used by FlatBuffer JSON output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FbKeyValue<K, V> {
    #[serde(alias = "Key")]
    pub key: K,
    #[serde(alias = "Value")]
    pub value: V,
}

/// Deserialize FlatBuffer's [{key, value}] array format into HashMap
pub fn deserialize_fb_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
{
    let items: Vec<FbKeyValue<K, V>> = Vec::deserialize(deserializer)?;
    Ok(items.into_iter().map(|kv| (kv.key, kv.value)).collect())
}

/// Deserialize FlatBuffer's [{key, value}] array format into HashMap, with default on missing
pub fn deserialize_fb_map_or_default<'de, D, K, V>(
    deserializer: D,
) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
{
    let items: Option<Vec<FbKeyValue<K, V>>> = Option::deserialize(deserializer)?;
    Ok(items
        .map(|v| v.into_iter().map(|kv| (kv.key, kv.value)).collect())
        .unwrap_or_default())
}

/// Deserialize a nested HashMap where both levels use FlatBuffer format
pub fn deserialize_fb_nested_map<'de, D, K1, K2, V>(
    deserializer: D,
) -> Result<HashMap<K1, HashMap<K2, V>>, D::Error>
where
    D: Deserializer<'de>,
    K1: Deserialize<'de> + std::hash::Hash + Eq,
    K2: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
{
    let items: Vec<FbKeyValue<K1, Vec<FbKeyValue<K2, V>>>> = Vec::deserialize(deserializer)?;
    Ok(items
        .into_iter()
        .map(|outer| {
            let inner_map = outer
                .value
                .into_iter()
                .map(|inner| (inner.key, inner.value))
                .collect();
            (outer.key, inner_map)
        })
        .collect())
}

/// Deserialize FlatBuffer's [{key, value}] array format into Option<HashMap>
pub fn deserialize_fb_map_option<'de, D, K, V>(
    deserializer: D,
) -> Result<Option<HashMap<K, V>>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
{
    let items: Option<Vec<FbKeyValue<K, V>>> = Option::deserialize(deserializer)?;
    Ok(items.map(|v| v.into_iter().map(|kv| (kv.key, kv.value)).collect()))
}
