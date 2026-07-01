//! On-demand stage **level** loading for the Stage Viewer.
//!
//! Resolves a stage id to its `level_id`, reads the matching `level_*.json` off
//! disk, and serves the raw Arknights level normalized for the frontend map
//! renderer: every object key is camelCased (EN/Yostar exports are `PascalCase`)
//! and `MapData.Map` is reshaped from the flattened `{Column_size, Matrix_data}`
//! form into a 2D grid. Results are cached so the file is parsed/transformed at
//! most once per stage.

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
