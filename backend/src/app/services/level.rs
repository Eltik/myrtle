//! Stage level loading for the Stage Viewer. Reads `level_*.json`, camelCases every
//! key (EN/Yostar exports are `PascalCase`), reshapes `MapData.Map` from the flat
//! `{Column_size, Matrix_data}` form into a 2D grid, and caches per stage.

use serde_json::Value;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;

pub async fn get_level(
    state: &AppState,
    server: Server,
    stage_id: &str,
) -> Result<Value, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let resource = format!("level:{stage_id}");
    let key = CacheKey::StaticData {
        resource: &resource,
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Value>(&key).await {
        return Ok(cached);
    }

    let gd = server_data.game_data.load_full();
    let level_id = match gd.stages.get(stage_id) {
        Some(stage) => stage.level_id.clone().ok_or(ApiError::NotFound)?,
        None => gd
            .mode_levels
            .get(stage_id)
            .cloned()
            .ok_or(ApiError::NotFound)?,
    };

    // level_id (e.g. "Obt/Main/level_main_01-07") lowercases to the on-disk path
    // under `gamedata/levels/`.
    let rel = level_id.to_lowercase().replace('\\', "/");
    let path = format!("{}/gamedata/levels/{rel}.json", server_data.assets_dir);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let mut raw: Value =
        serde_json::from_slice(&bytes).map_err(|e| ApiError::Internal(e.into()))?;
    reshape_map_matrix(&mut raw);
    let value = camelize_keys(raw);

    state.cache.set(&key, &value).await;
    Ok(value)
}

/// Rewrite `MapData.Map` from the EN/Yostar flattened `{Column_size, Row_size,
/// Matrix_data}` form into the row-major 2D grid (`number[][]`) the frontend
/// renderer expects. No-op if the field is already an array or is missing.
fn reshape_map_matrix(root: &mut Value) {
    let Some(map_data) = root.get_mut("MapData").and_then(Value::as_object_mut) else {
        return;
    };
    let grid = {
        let Some(map_obj) = map_data.get("Map").and_then(Value::as_object) else {
            return;
        };
        let cols = map_obj
            .get("Column_size")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        if cols == 0 {
            return;
        }
        let flat = map_obj
            .get("Matrix_data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        flat.chunks(cols)
            .map(|row| Value::Array(row.to_vec()))
            .collect::<Vec<Value>>()
    };
    map_data.insert("Map".to_string(), Value::Array(grid));
}

/// Recursively lowercase the first character of every object key, dropping a
/// trailing `_` first (EN exports name the reserved `type` field `Type_`). Turns
/// the `PascalCase` on-disk level into the camelCase shape the frontend consumes.
fn camelize_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (camel_key(&k), camelize_keys(v)))
                .collect(),
        ),
        Value::Array(arr) => Value::Array(arr.into_iter().map(camelize_keys).collect()),
        other => other,
    }
}

fn camel_key(key: &str) -> String {
    let key = key.strip_suffix('_').unwrap_or(key);
    let mut chars = key.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn camel_key_lowercases_the_first_char_and_drops_a_trailing_underscore() {
        assert_eq!(camel_key("MapData"), "mapData");
        assert_eq!(camel_key("Type_"), "type");
        assert_eq!(camel_key("alreadyCamel"), "alreadyCamel");
        assert_eq!(camel_key("ID"), "iD");
        assert_eq!(camel_key("_"), "");
        assert_eq!(camel_key(""), "");
        // Only one trailing underscore is stripped.
        assert_eq!(camel_key("Key__"), "key_");
    }

    #[test]
    fn camelize_keys_recurses_through_objects_and_arrays() {
        let raw = json!({
            "Options": { "CharacterLimit": 8, "Type_": "NORMAL" },
            "Waves": [ { "Fragments": [ { "Actions": [ { "Key": "enemy_1007_slime" } ] } ] } ],
            "Plain": "Value Stays",
        });
        assert_eq!(
            camelize_keys(raw),
            json!({
                "options": { "characterLimit": 8, "type": "NORMAL" },
                "waves": [ { "fragments": [ { "actions": [ { "key": "enemy_1007_slime" } ] } ] } ],
                "plain": "Value Stays",
            })
        );
        assert_eq!(camelize_keys(json!([1, "A", null])), json!([1, "A", null]));
    }

    #[test]
    fn reshape_turns_the_flat_matrix_into_rows() {
        let mut level = json!({ "MapData": { "Map": {
            "Column_size": 3, "Row_size": 2, "Matrix_data": [1, 2, 3, 4, 5, 6],
        }, "Other": true } });
        reshape_map_matrix(&mut level);
        assert_eq!(
            level,
            json!({ "MapData": { "Map": [[1, 2, 3], [4, 5, 6]], "Other": true } })
        );
    }

    #[test]
    fn reshape_keeps_a_ragged_tail_row() {
        let mut level = json!({ "MapData": { "Map": {
            "Column_size": 3, "Matrix_data": [1, 2, 3, 4],
        } } });
        reshape_map_matrix(&mut level);
        assert_eq!(level, json!({ "MapData": { "Map": [[1, 2, 3], [4]] } }));
    }

    #[test]
    fn reshape_without_matrix_data_yields_an_empty_grid() {
        let mut level = json!({ "MapData": { "Map": { "Column_size": 2 } } });
        reshape_map_matrix(&mut level);
        assert_eq!(level, json!({ "MapData": { "Map": [] } }));
    }

    #[test]
    fn reshape_is_a_no_op_when_there_is_nothing_to_reshape() {
        for original in [
            json!({ "MapData": { "Map": [[0, 1], [1, 0]] } }),
            json!({ "MapData": { "Map": { "Column_size": 0, "Matrix_data": [1] } } }),
            json!({ "MapData": { "Map": { "Matrix_data": [1] } } }),
            json!({ "MapData": "not an object" }),
            json!({ "Other": 1 }),
        ] {
            let mut level = original.clone();
            reshape_map_matrix(&mut level);
            assert_eq!(level, original);
        }
    }
}
