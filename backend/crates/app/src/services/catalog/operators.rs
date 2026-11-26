use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use myrtle_db::repos::stats;
use myrtle_engine_release::operator_changes::cn_changes as diff_cn;
use myrtle_engine_squad::rules::{SkillRules, module_ranks};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata::model::skin::SkinData;
use myrtle_gamedata_raw::tables::handbook::{OperatorBirthPlace, OperatorGender, OperatorRace};
use myrtle_gamedata_raw::tables::module::ModuleType;
use myrtle_gamedata_raw::tables::obtain::ObtainChannel;
use myrtle_gamedata_raw::tables::operator::{
    Operator, OperatorPosition, OperatorProfession, OperatorRarity,
};
use myrtle_gamedata_raw::tables::voice::Voices;
use myrtle_types::Server;
use serde::{Deserialize, Serialize};

use super::static_data::{TTL, cached, cached_heavy, key, now_rfc3339, to_json};
use crate::cache::{CachedJson, key_hash};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

pub const STATS_TTL: Duration = Duration::from_hours(1);
pub const MIN_SAMPLE: i64 = 50;

pub const ROSTER_FIELDS: [&str; 15] = [
    "id",
    "name",
    "profession",
    "professionName",
    "phases",
    "skills",
    "modules",
    "potentialRanks",
    "favorKeyFrames",
    "baseSkills",
    "skin",
    "portrait",
    "itemObtainApproach",
    "obtainChannel",
    "tmplDefault",
];

#[must_use]
pub fn ownership_key(server: Server) -> String {
    format!("operators:ownership:{}", server.as_str())
}

#[must_use]
pub fn build_stats_key(server: Server, operator_id: &str) -> String {
    format!("operators:buildstats:{}:{operator_id}", server.as_str())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexEntry {
    pub id: String,
    pub name: String,
    pub appellation: String,
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub sub_profession_id: String,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub nation_id: String,
    pub is_not_obtainable: bool,
    pub group_id: Option<String>,
    pub team_id: Option<String>,
    pub profession_name: Option<String>,
    pub sub_profession_name: Option<String>,
    pub nation_name: Option<String>,
    pub group_name: Option<String>,
    pub team_name: Option<String>,
    pub obtain_channel: Option<ObtainChannel>,
    pub artists: Vec<String>,
    pub portrait: Option<String>,
    pub gender: OperatorGender,
    pub race: OperatorRace,
    pub place_of_birth: OperatorBirthPlace,
    pub stats: OperatorIndexStats,
    pub has_offensive_recovery: bool,
    pub has_defensive_recovery: bool,
    pub all_skills_manual: bool,
    pub voice_actors: Vec<String>,
    pub phase_max_levels: Vec<i32>,
    pub skill_count: usize,
    pub potential_rank_count: usize,
    pub modules: Vec<OperatorIndexModule>,
    pub skill_rules: SkillRules,
    pub date_of_birth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexModule {
    pub uni_equip_id: String,
    pub type_name1: String,
    pub type_name2: Option<String>,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    pub rank_count: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexStats {
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub res: f64,
    pub cost: i32,
    pub block: i32,
}

#[must_use]
pub fn voice_actors(voices: &Voices, char_id: &str) -> Vec<String> {
    let Some(entry) = voices.voice_lang_dict.get(char_id) else {
        return Vec::new();
    };
    let mut langs: Vec<_> = entry.dict.iter().collect();
    langs.sort_unstable_by_key(|(key, _)| key.as_str());
    let mut seen = HashSet::new();
    langs
        .into_iter()
        .flat_map(|(_, lang)| &lang.cv_name)
        .filter(|name| seen.insert(name.as_str()))
        .cloned()
        .collect()
}

fn index_entry(id: &str, op: &Operator, voices: &Voices) -> OperatorIndexEntry {
    let id = op.id.clone().unwrap_or_else(|| id.to_owned());
    let stats = op
        .phases
        .last()
        .and_then(|p| p.attributes_key_frames.last())
        .map_or_else(OperatorIndexStats::default, |kf| OperatorIndexStats {
            hp: kf.data.max_hp,
            atk: kf.data.atk,
            def: kf.data.def,
            res: kf.data.magic_resistance,
            cost: kf.data.cost,
            block: kf.data.block_cnt,
        });
    let basic = op.profile.as_ref().map(|p| &p.basic_info);
    let sp_types: HashSet<&str> = op
        .skills
        .iter()
        .filter_map(|sk| sk.static_data.as_ref())
        .flat_map(|st| &st.levels)
        .map(|lvl| lvl.sp_data.sp_type.as_str())
        .collect();
    let all_skills_manual = !op.skills.is_empty()
        && op.skills.iter().all(|sk| {
            sk.static_data
                .as_ref()
                .and_then(|st| st.levels.first())
                .is_some_and(|lvl| matches!(lvl.skill_type.as_str(), "MANUAL" | "1"))
        });
    OperatorIndexEntry {
        voice_actors: voice_actors(voices, &id),
        id,
        name: op.name.clone(),
        appellation: op.appellation.clone(),
        rarity: rarity_to_stars(&op.rarity),
        profession: op.profession.clone(),
        sub_profession_id: op.sub_profession_id.clone(),
        position: op.position.clone(),
        tag_list: op.tag_list.clone(),
        nation_id: op.nation_id.clone(),
        is_not_obtainable: op.is_not_obtainable,
        group_id: op.group_id.clone(),
        team_id: op.team_id.clone(),
        profession_name: op.profession_name.clone(),
        sub_profession_name: op.sub_profession_name.clone(),
        nation_name: op.nation_name.clone(),
        group_name: op.group_name.clone(),
        team_name: op.team_name.clone(),
        obtain_channel: op.obtain_channel,
        artists: op.artists.clone(),
        portrait: op.portrait.clone(),
        gender: basic.map(|b| b.gender.clone()).unwrap_or_default(),
        race: basic.map(|b| b.race.clone()).unwrap_or_default(),
        place_of_birth: basic.map(|b| b.place_of_birth.clone()).unwrap_or_default(),
        stats,
        has_offensive_recovery: sp_types.contains("INCREASE_WHEN_ATTACK"),
        has_defensive_recovery: sp_types.contains("INCREASE_WHEN_TAKEN_DAMAGE"),
        all_skills_manual,
        phase_max_levels: op.phases.iter().map(|p| p.max_level).collect(),
        skill_count: op.skills.len(),
        potential_rank_count: op.potential_ranks.len(),
        modules: op
            .modules
            .iter()
            .map(|m| OperatorIndexModule {
                uni_equip_id: m.module.uni_equip_id.clone(),
                type_name1: m.module.type_name1.clone(),
                type_name2: m.module.type_name2.clone(),
                module_type: m.module.module_type.clone(),
                rank_count: module_ranks(m),
            })
            .collect(),
        skill_rules: SkillRules::of(op),
        date_of_birth: basic.map(|b| b.date_of_birth.clone()).unwrap_or_default(),
    }
}

fn index_entries(src: &GameModel, base: Option<&GameModel>) -> Vec<OperatorIndexEntry> {
    let mut entries: Vec<OperatorIndexEntry> = src
        .operators
        .iter()
        .filter(|(id, _)| base.is_none_or(|b| !b.operators.contains_key(*id)))
        .map(|(id, op)| index_entry(id, op, &src.voices))
        .collect();
    entries.sort_by(|a, b| {
        b.rarity
            .cmp(&a.rarity)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.id.cmp(&b.id))
    });
    entries
}

pub async fn index(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    cached(ctx, server, "operators_index", |gd| {
        to_json(&index_entries(gd, None))
    })
    .await
}

pub fn roster_row(op: &Operator) -> Result<serde_json::Value, AppError> {
    let mut row = serde_json::to_value(op).map_err(AppError::internal)?;
    if let serde_json::Value::Object(fields) = &mut row {
        fields.retain(|key, _| ROSTER_FIELDS.contains(&key.as_str()));
    }
    Ok(row)
}

pub fn roster_rows(
    operators: &HashMap<String, Operator>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let mut ids: Vec<&String> = operators.keys().collect();
    ids.sort_unstable();
    ids.into_iter()
        .map(|id| roster_row(&operators[id]))
        .collect()
}

pub async fn roster(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    cached_heavy(ctx, server, "operators_roster", "operators_roster", |gd| {
        to_json(&roster_rows(&gd.operators)?)
    })
    .await
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperatorOwnershipCounts {
    #[ts(type = "number")]
    pub owners: i64,
    #[ts(type = "number")]
    pub e2_owners: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperatorOwnershipResponse {
    #[ts(type = "number")]
    pub total_users: i64,
    pub counts: HashMap<String, OperatorOwnershipCounts>,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BuildChoice {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub skill_index: Option<i16>,
    #[ts(type = "number")]
    pub users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LevelBucket {
    pub level: i16,
    #[ts(type = "number")]
    pub users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SkillMasteryStats {
    pub skill_id: String,
    pub skill_index: i16,
    pub buckets: Vec<LevelBucket>,
    #[ts(type = "number")]
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLevelStats {
    pub uni_equip_id: String,
    pub buckets: Vec<LevelBucket>,
    #[ts(type = "number")]
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperatorBuildStatsResponse {
    pub operator_id: String,
    pub default_skills: Vec<BuildChoice>,
    #[ts(type = "number")]
    pub skill_total: i64,
    pub default_modules: Vec<BuildChoice>,
    #[ts(type = "number")]
    pub module_total: i64,
    pub masteries: Vec<SkillMasteryStats>,
    pub module_levels: Vec<ModuleLevelStats>,
    #[ts(type = "number")]
    pub min_sample: i64,
    pub computed_at: String,
}

#[must_use]
pub fn fill_levels(found: &[LevelBucket]) -> Vec<LevelBucket> {
    (0..=3)
        .map(|level| LevelBucket {
            level,
            users: found
                .iter()
                .find(|b| b.level == level)
                .map_or(0, |b| b.users),
        })
        .collect()
}

pub async fn ownership(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    ctx.snapshots.require(server)?;
    ctx.cache
        .json(&ownership_key(server), STATS_TTL, || async {
            let (total_users, rows) =
                stats::operator_ownership(&mut *ctx.db.conn().await?, server).await?;
            let counts = rows
                .into_iter()
                .map(|r| {
                    let counts = OperatorOwnershipCounts {
                        owners: r.owners,
                        e2_owners: r.e2_owners,
                    };
                    (r.operator_id, counts)
                })
                .collect();
            to_json(&OperatorOwnershipResponse {
                total_users,
                counts,
                computed_at: now_rfc3339(),
            })
        })
        .await
}

pub async fn build_stats(
    ctx: &AppContext,
    server: Server,
    operator_id: &str,
) -> Result<CachedJson, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    ctx.cache
        .json(
            &snapshot.key(&build_stats_key(server, operator_id)),
            STATS_TTL,
            || async move {
                let mut conn = ctx.db.conn().await?;
                let (skill_rows, module_rows) =
                    stats::operator_choices(&mut conn, server, operator_id).await?;
                let (mastery_rows, module_level_rows) =
                    stats::operator_levels(&mut conn, server, operator_id).await?;
                drop(conn);
                let skills = snapshot
                    .model
                    .operators
                    .get(operator_id)
                    .map(|op| op.skills.as_slice())
                    .unwrap_or_default();

                let default_skills: Vec<BuildChoice> = skill_rows
                    .into_iter()
                    .filter_map(|row| {
                        let index: i16 = row.choice.parse().ok()?;
                        let skill = skills.get(usize::try_from(index).ok()?)?;
                        Some(BuildChoice {
                            id: skill.skill_id.clone(),
                            skill_index: Some(index),
                            users: row.users,
                        })
                    })
                    .collect();
                let default_modules: Vec<BuildChoice> = module_rows
                    .into_iter()
                    .map(|row| BuildChoice {
                        id: row.choice,
                        skill_index: None,
                        users: row.users,
                    })
                    .collect();
                let skill_total: i64 = default_skills.iter().map(|c| c.users).sum();
                let module_total: i64 = default_modules.iter().map(|c| c.users).sum();

                let mut by_skill: HashMap<i16, Vec<LevelBucket>> = HashMap::new();
                for row in mastery_rows {
                    if let Ok(index) = row.key.parse::<i16>() {
                        by_skill.entry(index).or_default().push(LevelBucket {
                            level: row.level,
                            users: row.users,
                        });
                    }
                }
                let masteries = skills
                    .iter()
                    .enumerate()
                    .filter_map(|(index, skill)| {
                        let index = i16::try_from(index).ok()?;
                        let found = by_skill.get(&index)?;
                        let total: i64 = found.iter().map(|b| b.users).sum();
                        (total >= MIN_SAMPLE).then(|| SkillMasteryStats {
                            skill_id: skill.skill_id.clone(),
                            skill_index: index,
                            buckets: fill_levels(found),
                            total,
                        })
                    })
                    .collect();

                let mut by_module: HashMap<String, Vec<LevelBucket>> = HashMap::new();
                for row in module_level_rows {
                    by_module.entry(row.key).or_default().push(LevelBucket {
                        level: row.level,
                        users: row.users,
                    });
                }
                let module_levels = by_module
                    .into_iter()
                    .filter_map(|(uni_equip_id, found)| {
                        let total: i64 = found.iter().map(|b| b.users).sum();
                        (total >= MIN_SAMPLE).then(|| ModuleLevelStats {
                            uni_equip_id,
                            buckets: fill_levels(&found),
                            total,
                        })
                    })
                    .collect();

                to_json(&OperatorBuildStatsResponse {
                    operator_id: operator_id.to_owned(),
                    default_skills: if skill_total >= MIN_SAMPLE {
                        default_skills
                    } else {
                        Vec::new()
                    },
                    skill_total,
                    default_modules: if module_total >= MIN_SAMPLE {
                        default_modules
                    } else {
                        Vec::new()
                    },
                    module_total,
                    masteries,
                    module_levels,
                    min_sample: MIN_SAMPLE,
                    computed_at: now_rfc3339(),
                })
            },
        )
        .await
}

fn cn_pair(ctx: &AppContext, viewer: Server) -> Result<(Arc<Snapshot>, Arc<Snapshot>), AppError> {
    let cn = ctx.snapshots.require(Server::CN)?;
    let base = match viewer.data_server() {
        Server::CN => None,
        other => ctx.snapshots.get(other),
    };
    let base = match base {
        Some(base) => base,
        None => ctx.snapshots.or_default(ctx.snapshots.default_server())?,
    };
    Ok((cn, base))
}

fn cn_key(viewer: Server, resource: &str, cn: &Snapshot, base: &Snapshot) -> String {
    let version = &cn.model.meta.game_version;
    let fields = key_hash(format!("{}:{}", version.client, version.res).as_bytes());
    key(base, viewer, resource, &fields)
}

pub async fn upcoming(ctx: &AppContext, viewer: Server) -> Result<CachedJson, AppError> {
    let (cn, base) = cn_pair(ctx, viewer)?;
    ctx.cache
        .json(
            &cn_key(viewer, "upcoming", &cn, &base),
            TTL,
            || async move { to_json(&index_entries(&cn.model, Some(&base.model))) },
        )
        .await
}

pub async fn cn_changes(ctx: &AppContext, viewer: Server) -> Result<CachedJson, AppError> {
    let (cn, base) = cn_pair(ctx, viewer)?;
    if viewer.data_server() == Server::CN {
        return Ok(CachedJson::new("[]".to_owned()));
    }
    let cpu = ctx.cpu.clone();
    ctx.cache
        .json_detached(
            &cn_key(viewer, "cn_changes", &cn, &base),
            TTL,
            move || async move {
                cpu.run("cn_changes", move || {
                    to_json(&diff_cn(&base.model, &cn.model))
                })
                .await?
            },
        )
        .await
}

#[derive(Serialize)]
struct OperatorWithServer<'a> {
    #[serde(flatten)]
    operator: &'a Operator,
    server: &'a str,
}

fn with_server_json(op: &Operator, server: Server) -> Result<String, AppError> {
    to_json(&OperatorWithServer {
        operator: op,
        server: server.as_str(),
    })
}

pub async fn detail(ctx: &AppContext, server: Server, id: &str) -> Result<CachedJson, AppError> {
    cached(ctx, server, &format!("operator_detail:{id}"), |gd| {
        let op = gd.operators.get(id).ok_or(AppError::NotFound(None))?;
        with_server_json(op, server)
    })
    .await
}

#[must_use]
pub fn resolve_operator(
    ctx: &AppContext,
    preferred: Server,
    id: &str,
) -> Option<(Arc<Snapshot>, Server)> {
    std::iter::once(preferred)
        .chain(ctx.config.server.servers.iter().copied())
        .filter_map(|srv| Some((ctx.snapshots.get(srv)?, srv)))
        .find(|(snapshot, _)| snapshot.model.operators.contains_key(id))
}

pub async fn resolve_detail(
    ctx: &AppContext,
    preferred: Server,
    id: &str,
) -> Result<CachedJson, AppError> {
    let (snapshot, srv) = resolve_operator(ctx, preferred, id).ok_or(AppError::NotFound(None))?;
    let resource = format!("operator_resolve:{id}");
    ctx.cache
        .json(
            &key(&snapshot, preferred, &resource, srv.as_str()),
            TTL,
            || async {
                let op = snapshot
                    .model
                    .operators
                    .get(id)
                    .ok_or(AppError::NotFound(None))?;
                with_server_json(op, srv)
            },
        )
        .await
}

pub fn voices(ctx: &AppContext, server: Server, id: &str) -> Result<Voices, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    let voices = &snapshot.model.voices;
    Ok(Voices {
        char_words: voices
            .char_words
            .iter()
            .filter(|(_, v)| v.char_id == id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        voice_lang_dict: voices
            .voice_lang_dict
            .iter()
            .filter(|(_, vl)| vl.char_id == id)
            .map(|(k, vl)| (k.clone(), vl.clone()))
            .collect(),
        ..Voices::default()
    })
}

pub fn skins(ctx: &AppContext, server: Server, id: &str) -> Result<SkinData, AppError> {
    let snapshot = ctx.snapshots.require(server)?;
    Ok(SkinData {
        char_skins: snapshot
            .model
            .skins
            .char_skins
            .iter()
            .filter(|(_, s)| s.tmpl_id.as_deref().unwrap_or(&s.char_id) == id)
            .map(|(k, s)| (k.clone(), s.clone()))
            .collect(),
        ..SkinData::default()
    })
}

#[must_use]
pub const fn rarity_to_stars(rarity: &OperatorRarity) -> u8 {
    match rarity {
        OperatorRarity::SixStar => 6,
        OperatorRarity::FiveStar => 5,
        OperatorRarity::FourStar => 4,
        OperatorRarity::ThreeStar => 3,
        OperatorRarity::TwoStar => 2,
        OperatorRarity::OneStar => 1,
    }
}
