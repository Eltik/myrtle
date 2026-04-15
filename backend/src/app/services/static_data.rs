use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use serde_json::Value;

pub async fn get_resource(state: &AppState, resource: &str) -> Result<Value, ApiError> {
    let key = CacheKey::StaticData {
        resource,
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Value>(&key).await {
        return Ok(cached);
    }

    let gd = state.game_data.load();
    let value = serialize_resource(&gd, resource)?;
    state.cache.set(&key, &value).await;
    Ok(value)
}

fn serialize_resource(data: &GameData, resource: &str) -> Result<Value, ApiError> {
    let value = match resource {
        "operators" => serde_json::to_value(&data.operators),
        "skills" => serde_json::to_value(&data.skills),
        "modules" => serde_json::to_value(&data.modules),
        "skins" => serde_json::to_value(&data.skins),
        "materials" => serde_json::to_value(&data.materials),
        "stages" => serde_json::to_value(&data.stages),
        "zones" => serde_json::to_value(&data.zones),
        "enemies" => serde_json::to_value(&data.enemies),
        "gacha" => serde_json::to_value(&data.gacha),
        "voices" => serde_json::to_value(&data.voices),
        "handbook" => serde_json::to_value(&data.handbook),
        "chibis" => serde_json::to_value(&data.chibis),
        "trust" => serde_json::to_value(&data.favor),
        "ranges" => serde_json::to_value(&data.ranges),
        _ => return Err(ApiError::NotFound),
    };

    value.map_err(|e| ApiError::Internal(e.into()))
}
