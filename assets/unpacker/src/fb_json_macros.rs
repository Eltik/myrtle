//! Macros for FlatBuffer to JSON serialization
//!
//! This module provides declarative macros to convert FlatBuffer types to clean JSON,
//! matching the Python FBOHandler._to_json_dict() output format.

use serde_json::{json, Map, Value};

/// Trait for FlatBuffer types that can be serialized to JSON
pub trait FlatBufferToJson {
    fn to_json(&self) -> Value;
}

/// Convert field name to PascalCase for JSON output
/// Handles special cases like `def_` -> `Def_`, `trait_` -> `Trait_`
pub fn to_pascal_case(s: &str) -> String {
    // Handle trailing underscore (Rust keyword escape like `def_`, `trait_`)
    let (base, suffix) = if s.ends_with('_') && !s.ends_with("__") {
        (&s[..s.len() - 1], "_")
    } else {
        (s, "")
    };

    let mut result = String::new();
    let mut capitalize_next = true;

    for c in base.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result.push_str(suffix);
    result
}

/// Helper trait for enum types that have variant_name()
pub trait EnumToJson {
    fn to_json_value(&self) -> Value;
}

/// Macro to implement EnumToJson for FlatBuffer enum types
#[macro_export]
macro_rules! impl_enum_to_json {
    ($type:ty) => {
        impl $crate::fb_json_macros::EnumToJson for $type {
            fn to_json_value(&self) -> serde_json::Value {
                match self.variant_name() {
                    Some(name) => serde_json::json!(name),
                    None => serde_json::json!(format!("UNKNOWN_{}", self.0)),
                }
            }
        }
    };
}

/// Macro to add a single field to a JSON map
/// Supports different field types: string, scalar, enum, nested, vec_string, vec_nested, vec_scalar
#[macro_export]
macro_rules! fb_field {
    // Optional string field: Option<&str>
    ($map:expr, $name:expr, $value:expr, string) => {
        if let Some(v) = $value {
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                serde_json::json!(v),
            );
        }
    };

    // Required string field: &str (no Option)
    ($map:expr, $name:expr, $value:expr, string_required) => {
        $map.insert(
            $crate::fb_json_macros::to_pascal_case($name),
            serde_json::json!($value),
        );
    };

    // Scalar field: i32, f32, bool - always present with default
    ($map:expr, $name:expr, $value:expr, scalar) => {
        $map.insert(
            $crate::fb_json_macros::to_pascal_case($name),
            serde_json::json!($value),
        );
    };

    // Enum field with variant_name()
    ($map:expr, $name:expr, $value:expr, enum) => {
        $map.insert(
            $crate::fb_json_macros::to_pascal_case($name),
            $crate::fb_json_macros::EnumToJson::to_json_value(&$value),
        );
    };

    // Optional nested FlatBuffer type: Option<NestedType>
    ($map:expr, $name:expr, $value:expr, nested) => {
        if let Some(v) = $value {
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                $crate::fb_json_macros::FlatBufferToJson::to_json(&v),
            );
        }
    };

    // Vector of strings: Option<Vector<ForwardsUOffset<&str>>>
    ($map:expr, $name:expr, $value:expr, vec_string) => {
        if let Some(vec) = $value {
            let arr: Vec<serde_json::Value> = (0..vec.len())
                .map(|i| serde_json::json!(vec.get(i)))
                .collect();
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                serde_json::json!(arr),
            );
        }
    };

    // Vector of nested types: Option<Vector<ForwardsUOffset<NestedType>>>
    ($map:expr, $name:expr, $value:expr, vec_nested) => {
        if let Some(vec) = $value {
            let arr: Vec<serde_json::Value> = (0..vec.len())
                .map(|i| $crate::fb_json_macros::FlatBufferToJson::to_json(&vec.get(i)))
                .collect();
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                serde_json::json!(arr),
            );
        }
    };

    // Vector of key-value types that should become a JSON object
    ($map:expr, $name:expr, $value:expr, vec_kv) => {
        if let Some(vec) = $value {
            let mut kv_map = serde_json::Map::new();
            for i in 0..vec.len() {
                let entry = vec.get(i);
                let key = entry.key().to_string();
                if let Some(value) = entry.value() {
                    kv_map.insert(
                        key,
                        $crate::fb_json_macros::FlatBufferToJson::to_json(&value),
                    );
                }
            }
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                serde_json::Value::Object(kv_map),
            );
        }
    };

    // Vector of key-value types where value is a vector (array)
    ($map:expr, $name:expr, $value:expr, vec_kv_array) => {
        if let Some(vec) = $value {
            let mut kv_map = serde_json::Map::new();
            for i in 0..vec.len() {
                let entry = vec.get(i);
                let key = entry.key().to_string();
                if let Some(values) = entry.value() {
                    let arr: Vec<serde_json::Value> = (0..values.len())
                        .map(|j| $crate::fb_json_macros::FlatBufferToJson::to_json(&values.get(j)))
                        .collect();
                    kv_map.insert(key, serde_json::json!(arr));
                }
            }
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                serde_json::Value::Object(kv_map),
            );
        }
    };

    // Vector of scalars: Option<Vector<i32>> etc
    ($map:expr, $name:expr, $value:expr, vec_scalar) => {
        if let Some(vec) = $value {
            let arr: Vec<serde_json::Value> = vec.iter().map(|v| serde_json::json!(v)).collect();
            $map.insert(
                $crate::fb_json_macros::to_pascal_case($name),
                serde_json::json!(arr),
            );
        }
    };
}

/// Macro to implement FlatBufferToJson for a struct type
///
/// Usage:
/// ```ignore
/// impl_fb_to_json!(clz_Torappu_CharacterData<'_>,
///     name: string,
///     sortIndex: scalar,
///     rarity: enum,
///     phases: vec_nested,
/// );
/// ```
#[macro_export]
macro_rules! impl_fb_to_json {
    ($type:ty, $($field:ident : $kind:ident),* $(,)?) => {
        impl $crate::fb_json_macros::FlatBufferToJson for $type {
            fn to_json(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();
                $(
                    $crate::fb_field!(map, stringify!($field), self.$field(), $kind);
                )*
                serde_json::Value::Object(map)
            }
        }
    };
}

/// Macro to implement FlatBufferToJson for a dict type with key/value
#[macro_export]
macro_rules! impl_dict_to_json {
    ($type:ty) => {
        impl $crate::fb_json_macros::FlatBufferToJson for $type {
            fn to_json(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();
                map.insert("key".to_string(), serde_json::json!(self.key()));
                if let Some(value) = self.value() {
                    map.insert(
                        "value".to_string(),
                        $crate::fb_json_macros::FlatBufferToJson::to_json(&value),
                    );
                }
                serde_json::Value::Object(map)
            }
        }
    };
}
