use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use serde_json::Value;

pub async fn get_resource(
    state: &AppState,
    server: Server,
    resource: &str,
) -> Result<Value, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let key = CacheKey::StaticData {
        resource,
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Value>(&key).await {
        return Ok(cached);
    }

    let gd = server_data.game_data.load_full();
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
        "activities" => serde_json::to_value(&data.activities),
        "retro_acts" => serde_json::to_value(&data.retro_acts),
        "enemies" => serde_json::to_value(&data.enemies),
        "enemy-stages" => serde_json::to_value(&data.enemy_stage_index),
        "gacha" => serde_json::to_value(&data.gacha),
        "banners" => serde_json::to_value(&data.gacha.gacha_pool_client),
        "voices" => serde_json::to_value(&data.voices),
        "handbook" => serde_json::to_value(&data.handbook),
        "chibis" => serde_json::to_value(&data.chibis),
        "enemy-chibis" => serde_json::to_value(&data.enemy_chibis),
        "trust" => serde_json::to_value(&data.favor),
        "ranges" => serde_json::to_value(&data.ranges),
        _ => return Err(ApiError::NotFound),
    };

    value.map_err(|e| ApiError::Internal(e.into()))
}
