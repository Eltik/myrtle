use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use serde::Serialize;
use serde_json::Value;

use crate::app::cache::keys::CacheKey;
use crate::app::cache::{CachedJson, cached_json};
use crate::app::error::ApiError;
use crate::app::services::level::get_level;
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::chibi::ChibiCharacter;
use crate::core::gamedata::types::enemy::{Enemy, RaceData};
use crate::core::gamedata::types::enemy_stages::{EnemyStageRef, EnemyStageTable};
use crate::core::gamedata::types::material::Item;
use crate::core::gamedata::types::obtain::SkinChannel;
use crate::core::gamedata::types::skin::DisplaySkin;
use crate::core::gamedata::types::stage::{Stage, StageType};
use crate::core::gamedata::types::stage_index::StageIndexEntry;
use crate::core::gamedata::types::zone::{Zone, ZoneType};
use crate::core::hypergryph::constants::Server;

pub async fn get_resource(
    state: &AppState,
    server: Server,
    resource: &str,
) -> Result<CachedJson, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let key = cache_key(resource, server);
    cached_json(state, &key, move || async move {
        let gd = server_data.game_data.load_full();
        serialize_resource(&gd, resource)
    })
    .await
}

fn serialize_resource(data: &GameData, resource: &str) -> Result<String, ApiError> {
    let json = match resource {
        "operators" => serde_json::to_string(&data.operators),
        "skills" => serde_json::to_string(&data.skills),
        "modules" => serde_json::to_string(&data.modules),
        "skins" => serde_json::to_string(&data.skins),
        "materials" => serde_json::to_string(&data.materials),
        "stages" => serde_json::to_string(&data.stages),
        "zones" => serde_json::to_string(&data.zones),
        "activities" => serde_json::to_string(&data.activities),
        "retro_acts" => serde_json::to_string(&data.retro_acts),
        "enemies" => serde_json::to_string(&data.enemies),
        // The full per-appearance index. Nothing in this repo reads it after
        // the /enemies list moved to `enemy-stage-table`; it stays so a
        // frontend still running the old build keeps working across a deploy.
        "enemy-stages" => serde_json::to_string(&data.enemy_stage_index),
        "enemy-stage-table" => {
            serde_json::to_string(&EnemyStageTable::from_index(&data.enemy_stage_index))
        }
        "stage-index" => serde_json::to_string(&data.stage_index),
        "gacha" => serde_json::to_string(&data.gacha),
        "banners" => serde_json::to_string(&data.gacha.gacha_pool_client),
        "voices" => serde_json::to_string(&data.voices),
        "handbook" => serde_json::to_string(&data.handbook),
        "chibis" => serde_json::to_string(&data.chibis),
        "enemy-chibis" => serde_json::to_string(&data.enemy_chibis),
        "trust" => serde_json::to_string(&data.favor),
        "ranges" => serde_json::to_string(&data.ranges),
        _ => return Err(ApiError::NotFound),
    };

    json.map_err(|e| ApiError::Internal(e.into()))
}

const fn cache_key(resource: &str, server: Server) -> CacheKey<'_> {
    CacheKey::StaticData {
        resource,
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    }
}

// Stage detail: one stage with its zone, level, enemies and drops, instead of the full tables.

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StageDetailResponse<'a> {
    stage: Cow<'a, Stage>,
    zone: Option<Cow<'a, Zone>>,
    level_data: Option<Value>,
    /// Handbook records for every enemy referenced by the level (declared in
    /// `enemyDbRefs` or spawned by a wave action), keyed by enemy id.
    enemies: HashMap<&'a str, &'a Enemy>,
    /// Item records for every non-`CHAR` drop in `stageDropInfo`, keyed by id.
    materials: HashMap<&'a str, &'a Item>,
    /// True when `stage` and `zone` were built from the stage index for a
    /// procedural node, so the page can keep such nodes off the stage OG card.
    synthetic: bool,
}

/// Enemy ids referenced by a (camelCased) level: everything declared in
/// `enemyDbRefs[].id` plus every non-empty `waves[].fragments[].actions[].key`.
fn collect_level_enemy_ids(level: &Value) -> HashSet<String> {
    let mut ids = HashSet::new();
    if let Some(refs) = level.get("enemyDbRefs").and_then(Value::as_array) {
        for r in refs {
            if let Some(id) = r.get("id").and_then(Value::as_str) {
                ids.insert(id.to_owned());
            }
        }
    }
    if let Some(waves) = level.get("waves").and_then(Value::as_array) {
        for w in waves {
            let Some(frags) = w.get("fragments").and_then(Value::as_array) else {
                continue;
            };
            for f in frags {
                let Some(actions) = f.get("actions").and_then(Value::as_array) else {
                    continue;
                };
                for a in actions {
                    if let Some(key) = a.get("key").and_then(Value::as_str)
                        && !key.is_empty()
                    {
                        ids.insert(key.to_owned());
                    }
                }
            }
        }
    }
    ids
}

/// A stage and zone for a procedural IS / RA / CC / Paradox node, which has no
/// `stage_table` entry, built from its stage-index entry. Field for field what
/// the frontend's `syntheticStageFromIndex` built before this endpoint served
/// these nodes.
fn procedural_stage(entry: &StageIndexEntry) -> (Stage, Zone) {
    let stage = Stage {
        stage_id: entry.stage_id.clone(),
        level_id: entry.level_id.clone(),
        zone_id: entry.zone_id.clone(),
        code: entry.code.clone(),
        name: entry.name.clone(),
        stage_type: StageType::Activity,
        difficulty: entry.difficulty.clone(),
        ap_cost: entry.ap_cost,
        boss_mark: entry.boss,
        ..Stage::default()
    };
    let zone = Zone {
        zone_id: entry.zone_id.clone(),
        zone_index: i32::try_from(entry.zone_order).unwrap_or(i32::MAX),
        zone_type: ZoneType::Activity,
        zone_name_second: entry.zone_name.clone(),
        ..Zone::default()
    };
    (stage, zone)
}

pub async fn get_stage_detail(
    state: &AppState,
    server: Server,
    stage_id: &str,
) -> Result<CachedJson, ApiError> {
    state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let resource = format!("stage_detail:{stage_id}");
    let key = cache_key(&resource, server);
    cached_json(state, &key, move || async move {
        {
            let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;
            let gd = sd.game_data.load_full();
            if !gd.stages.contains_key(stage_id) && !gd.mode_levels.contains_key(stage_id) {
                return Err(ApiError::NotFound);
            }
        }

        // `get_level` caches on its own; `None` when the stage has no level file.
        let level_data = get_level(state, server, stage_id).await.ok();

        let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;
        let gd = sd.game_data.load_full();
        let synthetic = !gd.stages.contains_key(stage_id);
        let (stage, zone) = if let Some(stage) = gd.stages.get(stage_id) {
            (
                Cow::Borrowed(stage),
                gd.zones.get(&stage.zone_id).map(Cow::Borrowed),
            )
        } else {
            let entry = gd
                .stage_index
                .iter()
                .find(|e| e.stage_id == stage_id)
                .ok_or(ApiError::NotFound)?;
            let (stage, zone) = procedural_stage(entry);
            (Cow::Owned(stage), Some(Cow::Owned(zone)))
        };

        let mut enemies: HashMap<&str, &Enemy> = HashMap::new();
        if let Some(level) = &level_data {
            for id in collect_level_enemy_ids(level) {
                if let Some((k, e)) = gd.enemies.enemy_data.get_key_value(&id) {
                    enemies.insert(k.as_str(), e);
                }
            }
        }

        let mut materials: HashMap<&str, &Item> = HashMap::new();
        if let Some(drop_info) = &stage.stage_drop_info {
            for reward in &drop_info.display_detail_rewards {
                if reward.item_type == "CHAR" {
                    continue;
                }
                if let Some((k, item)) = gd.materials.items.get_key_value(&reward.id) {
                    materials.insert(k.as_str(), item);
                }
            }
        }

        serde_json::to_string(&StageDetailResponse {
            stage,
            zone,
            level_data,
            enemies,
            materials,
            synthetic,
        })
        .map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

// Enemy detail: the handbook record, the race table, and its "Appears In" stages.

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnemyDetailResponse<'a> {
    enemy: &'a Enemy,
    /// The full (small) race table; the page resolves the race name from
    /// `enemyTags` against it, as the old client join did.
    race_data: &'a HashMap<String, RaceData>,
}

pub async fn get_enemy_detail(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<CachedJson, ApiError> {
    let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let resource = format!("enemy_detail:{id}");
    let key = cache_key(&resource, server);
    cached_json(state, &key, move || async move {
        let gd = sd.game_data.load_full();
        let enemy = gd.enemies.enemy_data.get(id).ok_or(ApiError::NotFound)?;

        serde_json::to_string(&EnemyDetailResponse {
            enemy,
            race_data: &gd.enemies.race_data,
        })
        .map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

/// The "Appears In" list for one enemy: exactly `enemy_stage_index[id]` (empty
/// array when the enemy is not spawned anywhere), matching the per-enemy slice
/// the client used to pull from the full `/static/enemy-stages` map.
pub async fn get_enemy_stages(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<CachedJson, ApiError> {
    let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let resource = format!("enemy_stages:{id}");
    let key = cache_key(&resource, server);
    cached_json(state, &key, move || async move {
        let gd = sd.game_data.load_full();
        let empty: Vec<EnemyStageRef> = Vec::new();
        let refs = gd.enemy_stage_index.get(id).unwrap_or(&empty);

        serde_json::to_string(refs).map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

pub async fn get_chibi(
    state: &AppState,
    server: Server,
    operator_id: &str,
) -> Result<CachedJson, ApiError> {
    let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let resource = format!("chibi:{operator_id}");
    let key = cache_key(&resource, server);
    cached_json(state, &key, move || async move {
        let gd = sd.game_data.load_full();
        let entry: &ChibiCharacter = gd
            .chibis
            .get_by_operator(operator_id)
            .ok_or(ApiError::NotFound)?;

        serde_json::to_string(entry).map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

// Skins index: `skinId -> {charId, displaySkin}`, only the fields the profile Stats tab reads.

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SlimDisplaySkin<'a> {
    skin_name: Option<&'a str>,
    skin_group_id: &'a str,
    skin_group_name: &'a str,
    skin_group_sort_index: i32,
    display_tag_id: Option<&'a str>,
    get_time: i64,
    sort_id: i32,
    description: Option<&'a str>,
    content: &'a str,
    dialog: Option<&'a str>,
    usage: Option<&'a str>,
    obtain_approach: Option<&'a str>,
    designer_list: Option<&'a [String]>,
    drawer_list: &'a [String],
}

impl<'a> From<&'a DisplaySkin> for SlimDisplaySkin<'a> {
    fn from(d: &'a DisplaySkin) -> Self {
        Self {
            skin_name: d.skin_name.as_deref(),
            skin_group_id: &d.skin_group_id,
            skin_group_name: &d.skin_group_name,
            skin_group_sort_index: d.skin_group_sort_index,
            display_tag_id: d.display_tag_id.as_deref(),
            get_time: d.get_time,
            sort_id: d.sort_id,
            description: d.description.as_deref(),
            content: &d.content,
            dialog: d.dialog.as_deref(),
            usage: d.usage.as_deref(),
            obtain_approach: d.obtain_approach.as_deref(),
            designer_list: d.designer_list.as_deref(),
            drawer_list: &d.drawer_list,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SlimSkin<'a> {
    skin_id: &'a str,
    char_id: &'a str,
    display_skin: SlimDisplaySkin<'a>,
    /// `displayTagId` as a category, the same on every server.
    obtain_channel: SkinChannel,
}

pub async fn get_skins_index(state: &AppState, server: Server) -> Result<CachedJson, ApiError> {
    let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let key = cache_key("skins_index", server);
    cached_json(state, &key, move || async move {
        let gd = sd.game_data.load_full();
        let slim: HashMap<&str, SlimSkin> = gd
            .skins
            .char_skins
            .iter()
            .map(|(id, s)| {
                (
                    id.as_str(),
                    SlimSkin {
                        skin_id: &s.skin_id,
                        char_id: &s.char_id,
                        display_skin: SlimDisplaySkin::from(&s.display_skin),
                        obtain_channel: s.obtain_channel,
                    },
                )
            })
            .collect();

        serde_json::to_string(&slim).map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}
