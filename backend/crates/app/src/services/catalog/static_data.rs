use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use myrtle_db::repos::{progress, roster};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata::model::enemy_stages::EnemyStageTable;
use myrtle_gamedata::model::stage_index::StageIndexEntry;
use myrtle_gamedata_raw::tables::audio::EnemyAudio;
use myrtle_gamedata_raw::tables::character::DisplaySkin;
use myrtle_gamedata_raw::tables::enemy::{Enemy, RaceData};
use myrtle_gamedata_raw::tables::material::Item;
use myrtle_gamedata_raw::tables::obtain::SkinChannel;
use myrtle_gamedata_raw::tables::stage::{Stage, StageType};
use myrtle_gamedata_raw::tables::zone::{Zone, ZoneType};
use myrtle_types::Server;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::levels;
use crate::cache::CachedJson;
use crate::policy::Subject;
use crate::privacy::{self, layout::ProfileTabId};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

pub const TTL: Duration = Duration::from_hours(24);
pub const SKIN_POPULARITY_KEY: &str = "skins:popularity";
pub const SKIN_POPULARITY_TTL: Duration = Duration::from_hours(1);
pub const ENEMY_AVERAGE_KEY: &str = "enemies:community_average";
pub const ENEMY_AVERAGE_TTL: Duration = Duration::from_mins(30);

pub const RESOURCES: [&str; 21] = [
    "operators",
    "skills",
    "modules",
    "skins",
    "materials",
    "stages",
    "zones",
    "activities",
    "retro_acts",
    "enemies",
    "enemy-stages",
    "enemy-stage-table",
    "stage-index",
    "gacha",
    "banners",
    "voices",
    "handbook",
    "chibis",
    "enemy-chibis",
    "trust",
    "ranges",
];

#[must_use]
pub fn key(snapshot: &Snapshot, server: Server, resource: &str, fields: &str) -> String {
    snapshot.key(&format!(
        "static:{}:{resource}:{fields}",
        server.data_server().as_str()
    ))
}

pub(crate) fn to_json(value: &impl Serialize) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(AppError::internal)
}

pub(crate) async fn cached<F>(
    ctx: &AppContext,
    server: Server,
    resource: &str,
    build: F,
) -> Result<CachedJson, AppError>
where
    F: FnOnce(&GameModel) -> Result<String, AppError>,
{
    let snapshot = ctx.snapshots.require(server)?;
    ctx.cache
        .json(&key(&snapshot, server, resource, "0"), TTL, || async move {
            build(&snapshot.model)
        })
        .await
}

pub(crate) async fn cached_heavy<F>(
    ctx: &AppContext,
    server: Server,
    resource: &str,
    label: &'static str,
    build: F,
) -> Result<CachedJson, AppError>
where
    F: FnOnce(&GameModel) -> Result<String, AppError> + Send + 'static,
{
    let snapshot = ctx.snapshots.require(server)?;
    let key = key(&snapshot, server, resource, "0");
    let cpu = ctx.cpu.clone();
    ctx.cache
        .shared(&key, TTL, move || async move {
            cpu.run(label, move || build(&snapshot.model)).await?
        })
        .await
}

pub async fn resource(
    ctx: &AppContext,
    server: Server,
    name: &str,
) -> Result<CachedJson, AppError> {
    let name = *RESOURCES
        .iter()
        .find(|r| **r == name)
        .ok_or(AppError::NotFound(None))?;
    cached_heavy(ctx, server, name, "static_resource", move |gd| {
        serialize_resource(gd, name)
    })
    .await
}

pub fn serialize_resource(gd: &GameModel, name: &str) -> Result<String, AppError> {
    match name {
        "operators" => to_json(&gd.operators),
        "skills" => to_json(&gd.skills),
        "modules" => to_json(&gd.modules),
        "skins" => to_json(&gd.skins),
        "materials" => to_json(&gd.materials),
        "stages" => to_json(&gd.stages),
        "zones" => to_json(&gd.zones),
        "activities" => to_json(&gd.activities),
        "retro_acts" => to_json(&gd.retro_acts),
        "enemies" => to_json(&gd.enemies),
        "enemy-stages" => to_json(&gd.enemy_stage_index),
        "enemy-stage-table" => to_json(&EnemyStageTable::from_index(&gd.enemy_stage_index)),
        "stage-index" => to_json(&gd.stage_index),
        "gacha" => to_json(&gd.gacha),
        "banners" => to_json(&gd.gacha.gacha_pool_client),
        "voices" => to_json(&gd.voices),
        "handbook" => to_json(&gd.handbook),
        "chibis" => to_json(&gd.chibis),
        "enemy-chibis" => to_json(&gd.enemy_chibis),
        "trust" => to_json(&gd.favor),
        "ranges" => to_json(&gd.ranges),
        _ => Err(AppError::NotFound(None)),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StageDetailResponse<'a> {
    stage: Cow<'a, Stage>,
    zone: Option<Cow<'a, Zone>>,
    level_data: Option<Value>,
    enemies: HashMap<&'a str, &'a Enemy>,
    materials: HashMap<&'a str, &'a Item>,
    synthetic: bool,
}

#[must_use]
pub fn collect_level_enemy_ids(level: &Value) -> HashSet<String> {
    let refs = level
        .get("enemyDbRefs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("id").and_then(Value::as_str));
    let keys = level
        .get("waves")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|w| w.get("fragments").and_then(Value::as_array))
        .flatten()
        .filter_map(|f| f.get("actions").and_then(Value::as_array))
        .flatten()
        .filter_map(|a| a.get("key").and_then(Value::as_str))
        .filter(|key| !key.is_empty());
    refs.chain(keys).map(str::to_owned).collect()
}

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

pub async fn stage_detail(
    ctx: &AppContext,
    server: Server,
    stage_id: &str,
) -> Result<CachedJson, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    let resource = format!("stage_detail:{stage_id}");
    ctx.cache
        .json(&key(&snapshot, server, &resource, "0"), TTL, || async move {
            let gd = &snapshot.model;
            if !gd.stages.contains_key(stage_id) && !gd.mode_levels.contains_key(stage_id) {
                return Err(AppError::NotFound(None));
            }
            let level_data = match levels::read(ctx, gd, server, stage_id).await {
                Ok(level) => Some(level),
                Err(AppError::NotFound(_)) => None,
                Err(e) => {
                    tracing::warn!(server = server.as_str(), stage_id, error = %e, "stage served without its level");
                    None
                }
            };
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
                    .ok_or(AppError::NotFound(None))?;
                let (stage, zone) = procedural_stage(entry);
                (Cow::Owned(stage), Some(Cow::Owned(zone)))
            };
            let enemies = level_data
                .as_ref()
                .map(collect_level_enemy_ids)
                .unwrap_or_default()
                .iter()
                .filter_map(|id| gd.enemies.enemy_data.get_key_value(id))
                .map(|(k, e)| (k.as_str(), e))
                .collect();
            let materials = stage
                .stage_drop_info
                .iter()
                .flat_map(|d| &d.display_detail_rewards)
                .filter(|r| r.item_type != "CHAR")
                .filter_map(|r| gd.materials.items.get_key_value(&r.id))
                .map(|(k, item)| (k.as_str(), item))
                .collect();
            to_json(&StageDetailResponse {
                synthetic: !gd.stages.contains_key(stage_id),
                stage,
                zone,
                level_data,
                enemies,
                materials,
            })
        })
        .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnemyDetailResponse<'a> {
    enemy: &'a Enemy,
    race_data: &'a HashMap<String, RaceData>,
    audio: &'a [EnemyAudio],
}

pub async fn enemy_detail(
    ctx: &AppContext,
    server: Server,
    id: &str,
) -> Result<CachedJson, AppError> {
    cached(ctx, server, &format!("enemy_detail:{id}"), |gd| {
        let enemy = gd
            .enemies
            .enemy_data
            .get(id)
            .ok_or(AppError::NotFound(None))?;
        to_json(&EnemyDetailResponse {
            enemy,
            race_data: &gd.enemies.race_data,
            audio: gd.enemy_audio.get(id).map_or(&[], Vec::as_slice),
        })
    })
    .await
}

pub async fn enemy_stages(
    ctx: &AppContext,
    server: Server,
    id: &str,
) -> Result<CachedJson, AppError> {
    cached(ctx, server, &format!("enemy_stages:{id}"), |gd| {
        to_json(&gd.enemy_stage_index.get(id).map_or(&[][..], Vec::as_slice))
    })
    .await
}

pub async fn chibi(
    ctx: &AppContext,
    server: Server,
    operator_id: &str,
) -> Result<CachedJson, AppError> {
    cached(ctx, server, &format!("chibi:{operator_id}"), |gd| {
        to_json(
            gd.chibis
                .get_by_operator(operator_id)
                .ok_or(AppError::NotFound(None))?,
        )
    })
    .await
}

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
    obtain_channel: SkinChannel,
}

pub async fn skins_index(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    cached(ctx, server, "skins_index", |gd| {
        let slim: HashMap<&str, SlimSkin<'_>> = gd
            .skins
            .char_skins
            .iter()
            .map(|(id, s)| {
                let skin = SlimSkin {
                    skin_id: &s.skin_id,
                    char_id: &s.char_id,
                    display_skin: SlimDisplaySkin::from(&s.display_skin),
                    obtain_channel: s.obtain_channel,
                };
                (id.as_str(), skin)
            })
            .collect();
        to_json(&slim)
    })
    .await
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SkinPopularityResponse {
    #[ts(type = "number")]
    pub total_users: i64,
    #[ts(type = "Record<string, number>")]
    pub counts: HashMap<String, i64>,
    pub computed_at: String,
}

pub async fn skin_popularity(ctx: &AppContext) -> Result<CachedJson, AppError> {
    ctx.cache
        .json(SKIN_POPULARITY_KEY, SKIN_POPULARITY_TTL, || async {
            let (total_users, rows) = roster::skin_popularity(&mut *ctx.db.conn().await?).await?;
            to_json(&SkinPopularityResponse {
                total_users,
                counts: rows.into_iter().map(|r| (r.skin_id, r.owners)).collect(),
                computed_at: now_rfc3339(),
            })
        })
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommunityEnemyAverageResponse {
    pub average_encountered: f64,
    #[ts(type = "number")]
    pub user_count: i64,
    pub handbook_total: usize,
    pub computed_at: String,
}

pub async fn community_enemy_average(ctx: &AppContext) -> Result<CachedJson, AppError> {
    let snapshot = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    ctx.cache
        .json(
            &snapshot.key(ENEMY_AVERAGE_KEY),
            ENEMY_AVERAGE_TTL,
            || async move {
                let average =
                    progress::community_average_encountered(&mut *ctx.db.conn().await?).await?;
                let handbook_total = snapshot
                    .model
                    .enemies
                    .enemy_data
                    .values()
                    .filter(|e| !e.hide_in_handbook)
                    .count();
                to_json(&CommunityEnemyAverageResponse {
                    average_encountered: average.average,
                    user_count: average.users,
                    handbook_total,
                    computed_at: now_rfc3339(),
                })
            },
        )
        .await
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredEnemy {
    pub enemy_id: String,
    pub name: Option<String>,
    pub enemy_index: Option<String>,
    pub sort_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredEnemiesResponse {
    pub encountered_count: usize,
    pub handbook_total: usize,
    pub enemies: Vec<EncounteredEnemy>,
}

pub async fn encountered_enemies(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<EncounteredEnemiesResponse, AppError> {
    let user = privacy::resolve_user_id(ctx, subject, uid, &[ProfileTabId::Enemies]).await?;
    let ids = progress::encountered_enemies(&mut *ctx.db.conn().await?, user).await?;
    let snapshot = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    let handbook = &snapshot.model.enemies.enemy_data;
    let mut enemies: Vec<EncounteredEnemy> = ids
        .into_iter()
        .map(|enemy_id| {
            let entry = handbook.get(&enemy_id);
            EncounteredEnemy {
                name: entry.map(|e| e.name.clone()),
                enemy_index: entry.map(|e| e.enemy_index.clone()),
                sort_id: entry.map(|e| e.sort_id),
                enemy_id,
            }
        })
        .collect();
    enemies.sort_by(|a, b| {
        a.sort_id
            .is_none()
            .cmp(&b.sort_id.is_none())
            .then_with(|| a.sort_id.cmp(&b.sort_id))
            .then_with(|| a.enemy_id.cmp(&b.enemy_id))
    });
    Ok(EncounteredEnemiesResponse {
        encountered_count: enemies.len(),
        handbook_total: handbook.values().filter(|e| !e.hide_in_handbook).count(),
        enemies,
    })
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
