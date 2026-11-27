use std::path::{Path, PathBuf};

use myrtle_gamedata::model::GameModel;
use myrtle_types::Server;
use serde_json::Value;

use super::static_data::{TTL, key, to_json};
use crate::cache::CachedJson;
use crate::{AppContext, AppError};

pub async fn level(
    ctx: &AppContext,
    server: Server,
    stage_id: &str,
) -> Result<CachedJson, AppError> {
    if stage_id.contains(['/', '\\']) || stage_id.contains("..") {
        return Err(AppError::bad_request("invalid stage id"));
    }
    let snapshot = ctx.snapshots.require(server)?;
    let resource = format!("level:{stage_id}");
    ctx.cache
        .json(
            &key(&snapshot, server, &resource, "0"),
            TTL,
            || async move { to_json(&read(ctx, &snapshot.model, server, stage_id).await?) },
        )
        .await
}

pub(crate) async fn read(
    ctx: &AppContext,
    gd: &GameModel,
    server: Server,
    stage_id: &str,
) -> Result<Value, AppError> {
    let level_id = match gd.stages.get(stage_id) {
        Some(stage) => stage.level_id.as_deref(),
        None => gd.mode_levels.get(stage_id).map(String::as_str),
    }
    .ok_or(AppError::NotFound(None))?;
    let levels = ctx.snapshots.server_dir(server).join("gamedata/levels");
    let path = level_path(&levels, level_id).ok_or(AppError::NotFound(None))?;
    let bytes = tokio::fs::read(&path).await.map_err(read_error)?;
    let mut raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::Internal(format!("{}: {e}", path.display())))?;
    reshape_map_matrix(&mut raw);
    Ok(camelize_keys(raw))
}

#[must_use]
pub fn level_path(levels: &Path, level_id: &str) -> Option<PathBuf> {
    let rel = level_id.to_lowercase().replace('\\', "/");
    let safe = rel
        .split('/')
        .all(|part| !part.is_empty() && part != "." && part != ".." && !part.contains(':'));
    safe.then(|| levels.join(format!("{rel}.json")))
}

#[must_use]
pub fn read_error(e: std::io::Error) -> AppError {
    if e.kind() == std::io::ErrorKind::NotFound {
        AppError::NotFound(None)
    } else {
        AppError::Internal(format!("reading level file: {e}"))
    }
}

pub fn reshape_map_matrix(root: &mut Value) {
    let Some(map_data) = root.get_mut("MapData").and_then(Value::as_object_mut) else {
        return;
    };
    let Some(map) = map_data.get("Map").and_then(Value::as_object) else {
        return;
    };
    let cols = map
        .get("Column_size")
        .and_then(Value::as_u64)
        .and_then(|c| usize::try_from(c).ok())
        .unwrap_or(0);
    if cols == 0 {
        return;
    }
    let grid = map
        .get("Matrix_data")
        .and_then(Value::as_array)
        .map(|flat| {
            flat.chunks(cols)
                .map(|row| Value::Array(row.to_vec()))
                .collect()
        })
        .unwrap_or_default();
    map_data.insert("Map".to_owned(), Value::Array(grid));
}

#[must_use]
pub fn camelize_keys(value: Value) -> Value {
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

#[must_use]
pub fn camel_key(key: &str) -> String {
    let key = key.strip_suffix('_').unwrap_or(key);
    let mut chars = key.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_lowercase().to_string() + chars.as_str()
    })
}
