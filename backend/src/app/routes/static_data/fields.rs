use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Filter JSON object to only include specified fields
pub fn filter_fields<T: Serialize>(data: &T, fields: &HashSet<String>) -> Value {
    let full = serde_json::to_value(data).unwrap_or(Value::Null);

    match full {
        Value::Object(map) => {
            let filtered: Map<String, Value> = map
                .into_iter()
                .filter(|(k, _)| fields.contains(k))
                .collect();
            Value::Object(filtered)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| {
                    if let Value::Object(map) = v {
                        let filtered: Map<String, Value> = map
                            .into_iter()
                            .filter(|(k, _)| fields.contains(k))
                            .collect();
                        Value::Object(filtered)
                    } else {
                        v
                    }
                })
                .collect(),
        ),
        other => other,
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FieldsParam {
    pub fields: Option<String>, // Comma-separated: "id,name,rarity"
}

impl FieldsParam {
    pub fn to_set(&self) -> Option<HashSet<String>> {
        self.fields.as_ref().map(|f| {
            f.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }
}
