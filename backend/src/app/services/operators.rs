use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::cache::keys::CacheKey;
use crate::app::cache::{CachedJson, cached_json};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::handbook::{OperatorBirthPlace, OperatorGender, OperatorRace};
use crate::core::gamedata::types::module::ModuleType;
use crate::core::gamedata::types::obtain::ObtainChannel;
use crate::core::gamedata::types::operator::{
    Operator, OperatorPosition, OperatorProfession, OperatorRarity,
};
use crate::core::gamedata::types::skin::SkinData;
use crate::core::gamedata::types::voice::Voices;
use crate::core::hypergryph::constants::Server;
use crate::database::queries::operator_ownership;

/// Compact operator record for client-side search palettes and autocompletes.
/// Order of fields matches the JSON contract - keep stable.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexEntry {
    pub id: String,
    pub name: String,
    pub appellation: String,
    /// 1-6. Converts from the game's `TIER_N` enum so the frontend doesn't have to.
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub sub_profession_id: String,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub nation_id: String,
    pub is_not_obtainable: bool,
    /// Faction group id (list page "factions" filter + faction-logo fallback).
    pub group_id: Option<String>,
    /// Sub-faction / team id (same faction filter + logo fallback).
    pub team_id: Option<String>,
    /// The server's own names for `profession`, `subProfessionId`,
    /// `nationId`, `groupId` and `teamId`, so a client need not keep an
    /// English table per id. `None` when the server's table does not name the
    /// id.
    pub profession_name: Option<String>,
    pub sub_profession_name: Option<String>,
    pub nation_name: Option<String>,
    pub group_name: Option<String>,
    pub team_name: Option<String>,
    /// `itemObtainApproach` as a category, the same on every server.
    pub obtain_channel: Option<ObtainChannel>,
    /// Illustrator names (list page "artists" filter).
    pub artists: Vec<String>,
    /// Small portrait image path (grid card image src).
    pub portrait: Option<String>,
    /// From the operator profile - list page gender/race/birthplace filters and
    /// the randomizer "Feline" challenge (`race`). Defaulted when no profile.
    pub gender: OperatorGender,
    pub race: OperatorRace,
    pub place_of_birth: OperatorBirthPlace,
    /// Max-level, max-elite base stats, flattened so the list page can sort
    /// without pulling `phases`/`attributesKeyFrames`.
    pub stats: OperatorIndexStats,
    /// Randomizer challenge-filter booleans, precomputed so the randomizer
    /// doesn't need the full `/static/operators` table. `hasOffensiveRecovery` /
    /// `hasDefensiveRecovery`: any skill level with the matching `spType`.
    /// `allSkillsManual`: has >=1 skill and every skill's first level is manual.
    pub has_offensive_recovery: bool,
    pub has_defensive_recovery: bool,
    pub all_skills_manual: bool,
    /// Every `cvName` across the operator's voice languages, deduplicated, so
    /// the list page's voice-actor filter doesn't need the `/static/voices` table.
    pub voice_actors: Vec<String>,
    /// `maxLevel` per elite phase; its length is the phase count. With
    /// `skillCount`, `potentialRankCount` and `modules`, this is what the
    /// profile Stats tab reads instead of the `/static/operators` table.
    pub phase_max_levels: Vec<i32>,
    pub skill_count: usize,
    pub potential_rank_count: usize,
    pub modules: Vec<OperatorIndexModule>,
    /// The profile's `dateOfBirth` verbatim (e.g. "October 4"), empty when the
    /// operator has no profile. A plain string rather than an `Option` so a
    /// cached index from before this field fails to deserialize and is rebuilt.
    pub date_of_birth: String,
}

/// The module fields the profile Stats tab reads: id to match the roster, type
/// to pick the advanced ones, type names for the gap-list label.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexModule {
    pub uni_equip_id: String,
    pub type_name1: String,
    pub type_name2: Option<String>,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
}

/// Flattened final-phase base stats for the operators list sorters.
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexStats {
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub res: f64,
    pub cost: i32,
    pub block: i32,
}

/// Voice actors for `char_id`: every language's `cvName`, first occurrence wins.
/// Languages are walked in key order because the dict is a `HashMap`; the
/// frontend walked its serialized order, which was arbitrary per process.
fn voice_actors(voices: &Voices, char_id: &str) -> Vec<String> {
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

fn to_index_entry(id: &str, op: &Operator, voices: &Voices) -> OperatorIndexEntry {
    let id = op.id.clone().unwrap_or_else(|| id.to_string());
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

    let (gender, race, place_of_birth, date_of_birth) = op.profile.as_ref().map_or_else(
        <(OperatorGender, OperatorRace, OperatorBirthPlace, String)>::default,
        |pr| {
            (
                pr.basic_info.gender.clone(),
                pr.basic_info.race.clone(),
                pr.basic_info.place_of_birth.clone(),
                pr.basic_info.date_of_birth.clone(),
            )
        },
    );

    let mut has_offensive_recovery = false;
    let mut has_defensive_recovery = false;
    for sk in &op.skills {
        if let Some(st) = &sk.static_data {
            for lvl in &st.levels {
                match lvl.sp_data.sp_type.as_str() {
                    "INCREASE_WHEN_ATTACK" => has_offensive_recovery = true,
                    "INCREASE_WHEN_TAKEN_DAMAGE" => has_defensive_recovery = true,
                    _ => {}
                }
            }
        }
    }
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
        gender,
        race,
        place_of_birth,
        stats,
        has_offensive_recovery,
        has_defensive_recovery,
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
            })
            .collect(),
        date_of_birth,
    }
}

pub async fn get_index(
    state: &AppState,
    server: Server,
) -> Result<Vec<OperatorIndexEntry>, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let key = CacheKey::StaticData {
        resource: "operators_index",
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Vec<OperatorIndexEntry>>(&key).await {
        return Ok(cached);
    }

    let gd = server_data.game_data.load_full();
    let mut entries: Vec<OperatorIndexEntry> = gd
        .operators
        .iter()
        .map(|(id, op)| to_index_entry(id, op, &gd.voices))
        .collect();
    entries.sort_by(|a, b| b.rarity.cmp(&a.rarity).then_with(|| a.name.cmp(&b.name)));

    state.cache.set(&key, &entries).await;
    Ok(entries)
}

/// One operator's community counts. Struct-valued rather than a bare owner
/// tally so the next statistic is a field here instead of a fourth map
/// travelling alongside in the response.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorOwnershipCounts {
    #[ts(type = "number")]
    pub owners: i64,
    /// Owners who took this operator to E2.
    ///
    /// Zero is NOT a community signal for the 36 operators that cannot reach
    /// E2 at all: one to three stars carry fewer than three phases, so their
    /// rate is structurally zero. Clients must suppress the figure for those
    /// rather than rank them last.
    #[ts(type = "number")]
    pub e2_owners: i64,
}

/// Population-level ownership: how many sharing players own each operator, plus
/// the denominator. Only operators with at least one owner are listed; a missing
/// id implies zero owners. `totalUsers` is the eligible population on this
/// server (players who imported a roster and opted into stat sharing).
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorOwnershipResponse {
    #[ts(type = "number")]
    pub total_users: i64,
    pub counts: HashMap<String, OperatorOwnershipCounts>,
    pub computed_at: String,
}

/// One option in a community default distribution, with the count that put it
/// there. Ordered most-picked first by the query.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildChoice {
    /// `skillId` for a skill, `uniEquipId` for a module. Always a stable id:
    /// `skillIndex` is resolved here, against the same game data that produced
    /// the operator's `skills` array, so no client indexes by position.
    pub id: String,
    /// The game's 0-based skill index. Present for skills only, and carried for
    /// display ("S3") rather than for lookup.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub skill_index: Option<i16>,
    #[ts(type = "number")]
    pub users: i64,
}

/// One bucket of an investment histogram.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBucket {
    /// Mastery 0..3 for a skill, or module level 0..3 where 0 means the owner
    /// has not unlocked it. 0 is a real answer in both cases, and the largest
    /// bucket in most: it is where people stop, not missing data.
    pub level: i16,
    #[ts(type = "number")]
    pub users: i64,
}

/// How far E2 owners take one skill's mastery.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMasteryStats {
    /// Stable id, resolved from the stored index against the same game data
    /// that produces the operator's `skills` array.
    pub skill_id: String,
    pub skill_index: i16,
    /// Ascending by level, always covering 0..3 with zero-filled gaps so a
    /// client can render a fixed strip without reasoning about absence.
    pub buckets: Vec<LevelBucket>,
    #[ts(type = "number")]
    pub total: i64,
}

/// How far E2 owners take one module's level.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLevelStats {
    pub uni_equip_id: String,
    /// Ascending by level, zero-filled across 0..3.
    pub buckets: Vec<LevelBucket>,
    #[ts(type = "number")]
    pub total: i64,
}

/// What the community defaults to for one operator: which skill they leave
/// selected, and which module they leave equipped.
///
/// The distributions are whole, not just the winner, because 12 of 364
/// operators have a modal share under 50% and there the runner-up is as
/// informative as the leader. `skillTotal` and `moduleTotal` are the
/// denominators, and they differ: skills count E2 owners, modules count only
/// owners with an ADVANCED module actually equipped, which is 13.63% of rows.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorBuildStatsResponse {
    pub operator_id: String,
    pub default_skills: Vec<BuildChoice>,
    #[ts(type = "number")]
    pub skill_total: i64,
    pub default_modules: Vec<BuildChoice>,
    #[ts(type = "number")]
    pub module_total: i64,
    /// Mastery histogram per skill, and level histogram per module: where
    /// people STOP investing rather than what they picked. Empty below the
    /// reporting floor, like the distributions above.
    pub masteries: Vec<SkillMasteryStats>,
    pub module_levels: Vec<ModuleLevelStats>,
    /// Below this many samples a distribution is withheld entirely, so a rare
    /// operator's numbers cannot describe a handful of identifiable accounts.
    #[ts(type = "number")]
    pub min_sample: i64,
    pub computed_at: String,
}

/// Expand a sparse histogram into the fixed 0..3 strip, filling absent levels
/// with zero. The aggregate emits a row only where a user sits, so absence is
/// genuinely zero users rather than unknown, and collapsing the two here keeps
/// every client from re-deciding it.
fn fill_levels(found: &[LevelBucket]) -> Vec<LevelBucket> {
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

/// Withhold a distribution whose denominator is too thin to be either private
/// or meaningful. Costs 14 of 378 operators on skills and 147 of 370 on
/// modules; dropping it to 20 would recover those to 1 and 52.
const MIN_SAMPLE: i64 = 50;

/// Served from the precomputed aggregate via a short-lived cache, so a request
/// never scans the roster. Returns 404 for a server that is not loaded.
pub async fn get_ownership(
    state: &AppState,
    server: Server,
) -> Result<OperatorOwnershipResponse, ApiError> {
    state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let key = CacheKey::OperatorOwnership {
        server: server.as_str(),
    };
    if let Some(cached) = state.cache.get::<OperatorOwnershipResponse>(&key).await {
        return Ok(cached);
    }

    let server_id = server.index() as i16;
    let (total_users, rows) =
        operator_ownership::get_operator_ownership(&state.db, server_id).await?;
    let counts: HashMap<String, OperatorOwnershipCounts> = rows
        .into_iter()
        .map(|r| {
            (
                r.operator_id,
                OperatorOwnershipCounts {
                    owners: r.owners,
                    e2_owners: r.e2_owners,
                },
            )
        })
        .collect();

    let response = OperatorOwnershipResponse {
        total_users,
        counts,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    state.cache.set(&key, &response).await;
    Ok(response)
}

/// What the community defaults to for one operator.
///
/// Served from the precomputed distributions via a short-lived cache, so a
/// request never scans the roster. Returns 404 for a server that is not loaded,
/// and an empty distribution (not an error) for an operator nobody has built:
/// "no consensus yet" is a real answer and the client falls back to its own
/// default.
pub async fn get_build_stats(
    state: &AppState,
    server: Server,
    operator_id: &str,
) -> Result<OperatorBuildStatsResponse, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let key = CacheKey::OperatorBuildStats {
        server: server.as_str(),
        operator_id,
    };
    if let Some(cached) = state.cache.get::<OperatorBuildStatsResponse>(&key).await {
        return Ok(cached);
    }

    let server_id = server.index() as i16;
    let (skill_rows, module_rows) =
        operator_ownership::get_operator_choices(&state.db, server_id, operator_id).await?;
    let (mastery_rows, module_level_rows) =
        operator_ownership::get_operator_levels(&state.db, server_id, operator_id).await?;

    let game_data = server_data.game_data.load();
    let skills = game_data
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

    let (default_skills, default_modules) = (
        if skill_total >= MIN_SAMPLE {
            default_skills
        } else {
            Vec::new()
        },
        if module_total >= MIN_SAMPLE {
            default_modules
        } else {
            Vec::new()
        },
    );

    let mut mastery_by_index: HashMap<i16, Vec<LevelBucket>> = HashMap::new();
    for row in mastery_rows {
        if let Ok(index) = row.key.parse::<i16>() {
            mastery_by_index
                .entry(index)
                .or_default()
                .push(LevelBucket {
                    level: row.level,
                    users: row.users,
                });
        }
    }
    let masteries: Vec<SkillMasteryStats> = skills
        .iter()
        .enumerate()
        .filter_map(|(index, skill)| {
            let index = i16::try_from(index).ok()?;
            let found = mastery_by_index.get(&index)?;
            let total: i64 = found.iter().map(|b| b.users).sum();
            (total >= MIN_SAMPLE).then(|| SkillMasteryStats {
                skill_id: skill.skill_id.clone(),
                skill_index: index,
                buckets: fill_levels(found),
                total,
            })
        })
        .collect();

    let mut levels_by_module: HashMap<String, Vec<LevelBucket>> = HashMap::new();
    for row in module_level_rows {
        levels_by_module
            .entry(row.key)
            .or_default()
            .push(LevelBucket {
                level: row.level,
                users: row.users,
            });
    }
    let module_levels: Vec<ModuleLevelStats> = levels_by_module
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

    let response = OperatorBuildStatsResponse {
        operator_id: operator_id.to_owned(),
        default_skills,
        skill_total,
        default_modules,
        module_total,
        masteries,
        module_levels,
        min_sample: MIN_SAMPLE,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    state.cache.set(&key, &response).await;
    Ok(response)
}

/// Operators present on `source` but absent from the default (global/EN) server:
/// the "upcoming operators" preview. Returns 404 when `source` is not a loaded
/// server.
pub async fn get_upcoming(
    state: &AppState,
    source: Server,
) -> Result<Vec<OperatorIndexEntry>, ApiError> {
    let source_data = state.try_server_data(source).ok_or(ApiError::NotFound)?;
    let key = CacheKey::StaticData {
        resource: "upcoming",
        server: source.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Vec<OperatorIndexEntry>>(&key).await {
        return Ok(cached);
    }

    let base = state.default_game_data();
    let src = source_data.game_data.load_full();
    let base_ids: HashSet<&str> = base.operators.keys().map(String::as_str).collect();

    let mut entries: Vec<OperatorIndexEntry> = src
        .operators
        .iter()
        .filter(|(id, _)| !base_ids.contains(id.as_str()))
        .map(|(id, op)| to_index_entry(id, op, &src.voices))
        .collect();
    entries.sort_by(|a, b| b.rarity.cmp(&a.rarity).then_with(|| a.name.cmp(&b.name)));

    state.cache.set(&key, &entries).await;
    Ok(entries)
}

#[derive(Serialize)]
struct OperatorWithServer<'a> {
    #[serde(flatten)]
    operator: &'a Operator,
    server: &'a str,
}

fn operator_with_server_json(op: &Operator, server: Server) -> Result<String, ApiError> {
    serde_json::to_string(&OperatorWithServer {
        operator: op,
        server: server.as_str(),
    })
    .map_err(|e| ApiError::Internal(e.into()))
}

/// Loaded servers in resolution order: `preferred` first, then the rest of the
/// configured servers. Shared by `resolve_operator` and `resolve_operator_json`.
fn server_order(state: &AppState, preferred: Server) -> Vec<Server> {
    let mut order = vec![preferred];
    order.extend(
        state
            .config
            .servers
            .iter()
            .copied()
            .filter(|s| *s != preferred),
    );
    order
}

/// One fully-enriched operator by id, as pre-serialized JSON (+ its `ETag`), cached
/// per `(server, id)`. Lets the detail page fetch a single operator instead of
/// downloading the whole `/static/operators` table.
pub async fn get_operator_json(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<CachedJson, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let resource = format!("operator_detail:{id}");
    let key = CacheKey::StaticData {
        resource: &resource,
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    cached_json(state, &key, move || async move {
        let gd = server_data.game_data.load_full();
        let op = gd.operators.get(id).ok_or(ApiError::NotFound)?;
        operator_with_server_json(op, server)
    })
    .await
}

/// Find an operator across loaded servers, `preferred` first, and the server it was
/// found on, so one request resolves both global and CN-only operators.
pub fn resolve_operator(
    state: &AppState,
    preferred: Server,
    id: &str,
) -> Option<(Operator, Server)> {
    for srv in server_order(state, preferred) {
        if let Some(sd) = state.try_server_data(srv)
            && let Some(op) = sd.game_data.load_full().operators.get(id)
        {
            return Some((op.clone(), srv));
        }
    }
    None
}

/// Resolve one operator across loaded servers (preferring `preferred`) as
/// pre-serialized JSON (+ its `ETag`). Cached on `preferred` with a distinct prefix,
/// since resolution is deterministic from `preferred` + `config.servers`. Returns
/// `ApiError::NotFound` when the id resolves on no loaded server.
pub async fn resolve_operator_json(
    state: &AppState,
    preferred: Server,
    id: &str,
) -> Result<CachedJson, ApiError> {
    let resource = format!("operator_resolve:{id}");
    let key = CacheKey::StaticData {
        resource: &resource,
        server: preferred.as_str(),
        fields_hash: 0,
        page: 0,
    };
    cached_json(state, &key, move || async move {
        for srv in server_order(state, preferred) {
            if let Some(sd) = state.try_server_data(srv) {
                let gd = sd.game_data.load_full();
                if let Some(op) = gd.operators.get(id) {
                    return operator_with_server_json(op, srv);
                }
            }
        }
        Err(ApiError::NotFound)
    })
    .await
}

/// Just one operator's voice lines (+ its voice-lang entry), so the Audio tab
/// fetches KB instead of the whole `/static/voices` table.
pub async fn get_operator_voices(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<Voices, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    Ok(Voices {
        char_words: gd
            .voices
            .char_words
            .iter()
            .filter(|(_, v)| v.char_id.as_str() == id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        voice_lang_dict: gd
            .voices
            .voice_lang_dict
            .iter()
            .filter(|(_, vl)| vl.char_id.as_str() == id)
            .map(|(k, vl)| (k.clone(), vl.clone()))
            .collect(),
        ..Default::default()
    })
}

/// Just one operator's skins, so the Skins tab fetches KB instead of the whole
/// `/static/skins` table. Matches by `tmplId` (alternate forms) else `charId`.
pub async fn get_operator_skins(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<SkinData, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    Ok(SkinData {
        char_skins: gd
            .skins
            .char_skins
            .iter()
            .filter(|(_, s)| match &s.tmpl_id {
                Some(t) => t.as_str() == id,
                None => s.char_id.as_str() == id,
            })
            .map(|(k, s)| (k.clone(), s.clone()))
            .collect(),
        ..Default::default()
    })
}

/// 1-6 stars from the game's `TIER_N` enum. The story index carries the
/// same representation on an operator's record group.
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::voice_actors;
    use crate::core::gamedata::types::voice::{LangType, VoiceLang, VoiceLangDictEntry, Voices};

    fn lang(voice_lang_type: LangType, cv_name: &[&str]) -> VoiceLangDictEntry {
        VoiceLangDictEntry {
            voice_lang_type,
            cv_name: cv_name.iter().map(|s| (*s).to_string()).collect(),
            ..Default::default()
        }
    }

    // Same input the TS `extractVoiceActors` was checked against: it yields
    // ["Hanazawa Kana", "Wang Yaxin", "Li Jiaxin", "Kana Hanazawa"].
    #[test]
    fn voice_actors_dedupe_across_languages_in_key_order() {
        let dict = HashMap::from([
            ("JP".to_string(), lang(LangType::Jp, &["Hanazawa Kana"])),
            (
                "CN_MANDARIN".to_string(),
                lang(LangType::CnMandarin, &["Hanazawa Kana", "Wang Yaxin"]),
            ),
            (
                "EN".to_string(),
                lang(LangType::En, &["Li Jiaxin", "Kana Hanazawa", "Li Jiaxin"]),
            ),
        ]);
        let voices = Voices {
            voice_lang_dict: HashMap::from([(
                "char_002_amiya".to_string(),
                VoiceLang {
                    char_id: "char_002_amiya".to_string(),
                    dict,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };

        assert_eq!(
            voice_actors(&voices, "char_002_amiya"),
            ["Hanazawa Kana", "Wang Yaxin", "Li Jiaxin", "Kana Hanazawa"]
        );
        assert!(voice_actors(&voices, "char_999_none").is_empty());
    }
}
