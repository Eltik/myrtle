use chrono::Utc;
use serde::Deserialize;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::{
        grade::calculate::calculate_user_grade,
        hypergryph::{
            constants::{AuthSession, Server},
            yostar::sync_data_raw,
        },
    },
    database::{
        models::score::UserScore,
        queries::{roster, score, users},
    },
};

#[derive(Deserialize)]
pub struct SyncDataResponse {
    pub result: Option<i32>,
    pub user: Option<GameUser>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameUser {
    pub status: Option<PlayerStatus>,
    pub troop: Option<Troop>,
    pub inventory: Option<serde_json::Map<String, serde_json::Value>>,
    pub skin: Option<SkinStore>,
    pub medal: Option<MedalStore>,

    pub dungeon: Option<serde_json::Value>,
    pub building: Option<serde_json::Value>,
    #[serde(rename = "rlv2")]
    pub roguelike: Option<serde_json::Value>,
    #[serde(rename = "deepSea")]
    pub sandbox: Option<serde_json::Value>,
    #[serde(rename = "sandboxPerm")]
    pub sandbox_perm: Option<serde_json::Value>,
    #[serde(rename = "checkIn")]
    pub checkin: Option<CheckIn>,
    pub social: Option<Social>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Social {
    /// Up to 3 entries (some may be `null` for empty slots) — references
    /// troop slots by `charInstId`. Resolved to operator_id in
    /// `extract_supports`.
    pub assist_char_list: Option<Vec<Option<AssistChar>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistChar {
    pub char_inst_id: Option<i64>,
    pub skill_index: Option<i64>,
    pub current_equip: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatus {
    #[serde(rename = "nickName")]
    pub nick_name: Option<String>,
    pub level: Option<i64>,
    pub avatar: Option<Avatar>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume: Option<String>,
    pub exp: Option<i64>,
    #[serde(rename = "diamondShard")]
    pub orundum: Option<i64>,
    pub gold: Option<i64>,
    pub ap: Option<i64>,
    pub max_ap: Option<i64>,
    pub gacha_ticket: Option<i64>,
    pub ten_gacha_ticket: Option<i64>,
    pub classic_gacha_ticket: Option<i64>,
    pub classic_ten_gacha_ticket: Option<i64>,
    pub recruit_license: Option<i64>,
    pub social_point: Option<i64>,
    pub hgg_shard: Option<i64>,
    pub lgg_shard: Option<i64>,
    pub practice_ticket: Option<i64>,
    #[serde(rename = "monthlySubscriptionEndTime")]
    pub monthly_sub_end: Option<i64>,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub friend_num_limit: Option<i64>,
    pub main_stage_progress: Option<String>,
}

#[derive(Deserialize)]
pub struct Avatar {
    pub id: Option<String>,
}

#[derive(Deserialize)]
pub struct Troop {
    pub chars: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TroopChar {
    pub char_id: Option<String>,
    pub evolve_phase: Option<i64>,
    pub level: Option<i64>,
    pub exp: Option<i64>,
    pub potential_rank: Option<i64>,
    pub main_skill_lvl: Option<i64>,
    pub favor_point: Option<i64>,
    pub skin: Option<String>,
    pub default_skill_index: Option<i64>,
    pub current_equip: Option<String>,
    pub gain_time: Option<i64>,
    pub voice_lan: Option<String>,
    pub current_tmpl: Option<String>,
    pub skills: Option<Vec<TroopSkill>>,
    pub equip: Option<serde_json::Map<String, serde_json::Value>>,
    /// Amiya stores per-form data under `tmpl[<form_id>]` instead of mirroring
    /// it at the top level. Each entry has its own elite/level/skills/equip/skin.
    /// `None` for normal operators.
    pub tmpl: Option<std::collections::HashMap<String, TroopTmpl>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TroopTmpl {
    pub skin_id: Option<String>,
    pub default_skill_index: Option<i64>,
    pub current_equip: Option<String>,
    pub skills: Option<Vec<TroopSkill>>,
    pub equip: Option<serde_json::Map<String, serde_json::Value>>,
    // Per-form progression (Hypergryph sometimes stores these inside the tmpl
    // entry; fall back to the parent TroopChar fields when missing).
    pub evolve_phase: Option<i64>,
    pub level: Option<i64>,
    pub exp: Option<i64>,
    pub main_skill_lvl: Option<i64>,
    pub favor_point: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TroopSkill {
    pub specialize_level: Option<i64>,
}

#[derive(Deserialize)]
pub struct EquipEntry {
    pub level: Option<i64>,
    pub locked: Option<i64>, // 0 / 1 integer
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinStore {
    pub character_skins: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinEntry {
    pub obtained_at: Option<i64>,
}

#[derive(Deserialize)]
pub struct MedalStore {
    pub medals: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Deserialize)]
pub struct CheckIn {
    pub history: Option<Vec<i16>>,
}

pub async fn refresh(
    state: &AppState,
    user_id: &str,
    server: Server,
) -> Result<serde_json::Value, ApiError> {
    let session_json: Option<String> = state
        .cache
        .get(&CacheKey::GameSession { uid: user_id })
        .await;

    let session_json =
        session_json.ok_or(ApiError::BadRequest("no game session — login again".into()))?;

    let mut session: AuthSession = serde_json::from_str(&session_json)
        .map_err(|_| ApiError::BadRequest("invalid game session".into()))?;

    let text = match sync_data_raw(&state.http_client, &mut session, server).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(
                uid = %user_id,
                server = server.as_str(),
                error = ?e,
                "yostar refresh (account/syncData) failed"
            );
            return Err(e.into());
        }
    };

    let data: SyncDataResponse =
        serde_json::from_str(&text).map_err(|e| ApiError::Internal(e.into()))?;

    let raw: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| ApiError::Internal(e.into()))?;

    let user = data
        .user
        .ok_or(ApiError::BadRequest("missing user data in response".into()))?;

    let status = user.status.as_ref();

    let nickname = status.and_then(|s| s.nick_name.as_deref()).unwrap_or("");
    // nickNumber comes from Hypergryph as either a string ("1234") or an
    // integer (1234) depending on server/version, and may live on
    // `status` or as a sibling of it. Probe both paths and accept either
    // type. Stored as string so leading zeros survive.
    let nick_number_owned = extract_nick_number(&raw);
    let nick_number = nick_number_owned.as_deref();
    let level = status.and_then(|s| s.level).unwrap_or(0) as i16;
    let avatar_id = status
        .and_then(|s| s.avatar.as_ref())
        .and_then(|a| a.id.as_deref());
    let secretary = status.and_then(|s| s.secretary.as_deref());
    let secretary_skin_id = status.and_then(|s| s.secretary_skin_id.as_deref());
    let resume_id = status.and_then(|s| s.resume.as_deref());

    let operators = extract_operators(&user.troop);
    let skills = extract_skills(&user.troop);
    let modules = extract_modules(&user.troop);
    let items = extract_items(&user.inventory);
    let skins = extract_skins(&user.skin);
    let status_json = extract_status(status);
    let supports = extract_supports(&user.troop, &user.social);
    let stages = user
        .dungeon
        .as_ref()
        .and_then(|d| d.get("stages"))
        .cloned()
        .unwrap_or_default();
    let roguelike = extract_roguelike(&user.roguelike);
    let sandbox = user.sandbox_perm.unwrap_or_default();
    let medals = extract_medals(&user.medal);
    let building = user.building.unwrap_or_default();
    let checkin: Vec<i16> = user.checkin.and_then(|c| c.history).unwrap_or_default();

    roster::sync_user_data(
        &state.db,
        user_id,
        server.index() as i16,
        nickname,
        level,
        avatar_id,
        secretary,
        secretary_skin_id,
        resume_id,
        &operators,
        &skills,
        &modules,
        &items,
        &skins,
        &status_json,
        &stages,
        &roguelike,
        &sandbox,
        &medals,
        &building,
        &checkin,
        &supports,
        nick_number,
    )
    .await?;

    if let Some(user) = users::find_by_uid(&state.db, user_id).await? {
        let grade = calculate_user_grade(&state.db, user.id, &state.game_data.load()).await?;

        score::update_score(
            &state.db,
            &UserScore {
                user_id: user.id,
                operator_score: grade.operator_grade,
                total_score: grade.total_score, // for now, just operator
                grade: Some(grade.overall),
                stage_score: grade.stage_grade,
                roguelike_score: grade.roguelike_grade,
                sandbox_score: grade.sandbox_grade,
                medal_score: grade.medal_grade,
                base_score: grade.base_grade,
                skin_score: 0.0,
                calculated_at: Utc::now(),
            },
        )
        .await?;
    }

    Ok(raw)
}

/// Pull `nickNumber` out of the raw syncData. Tolerates string,
/// integer, or absent values, and probes the few paths Arknights has
/// been observed to put it under.
fn extract_nick_number(raw: &serde_json::Value) -> Option<String> {
    let candidates = [
        raw.pointer("/user/status/nickNumber"),
        raw.pointer("/user/social/nickNumber"),
        raw.pointer("/user/nickNumber"),
    ];

    for candidate in candidates.into_iter().flatten() {
        let value = match candidate {
            serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        };
        if value.is_some() {
            return value;
        }
    }
    None
}

fn extract_operators(troop: &Option<Troop>) -> serde_json::Value {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return serde_json::json!([]);
    };

    let mut ops: Vec<serde_json::Value> = Vec::new();
    for raw in chars.values() {
        let Ok(c) = serde_json::from_value::<TroopChar>(raw.clone()) else {
            continue;
        };
        let Some(char_id) = c.char_id.clone() else {
            continue;
        };

        // Amiya: emit one row per form. The base char_id row uses the same
        // id as the original troop key (e.g. `char_002_amiya`) so existing
        // lookups still resolve. Branches (`char_1001_amiya2`,
        // `char_1037_amiya3`) get their own rows with per-form progression.
        if let Some(tmpl) = c.tmpl.as_ref()
            && !tmpl.is_empty()
        {
            for (form_id, form) in tmpl {
                ops.push(serde_json::json!({
                    "operator_id": form_id,
                    "elite": form.evolve_phase.or(c.evolve_phase).unwrap_or(0),
                    "level": form.level.or(c.level).unwrap_or(1),
                    "exp": form.exp.or(c.exp).unwrap_or(0),
                    "potential": c.potential_rank.unwrap_or(0),
                    "skill_level": form.main_skill_lvl.or(c.main_skill_lvl).unwrap_or(1),
                    "favor_point": form.favor_point.or(c.favor_point).unwrap_or(0),
                    "skin_id": form.skin_id.clone().or_else(|| c.skin.clone()).unwrap_or_default(),
                    "default_skill": form.default_skill_index.or(c.default_skill_index).unwrap_or(0),
                    "current_equip": form.current_equip.clone().or_else(|| c.current_equip.clone()),
                    "obtained_at": c.gain_time.unwrap_or(0),
                    "voice_lan": c.voice_lan.clone(),
                    "current_tmpl": c.current_tmpl.clone(),
                }));
            }
            continue;
        }

        ops.push(serde_json::json!({
            "operator_id": char_id,
            "elite": c.evolve_phase.unwrap_or(0),
            "level": c.level.unwrap_or(1),
            "exp": c.exp.unwrap_or(0),
            "potential": c.potential_rank.unwrap_or(0),
            "skill_level": c.main_skill_lvl.unwrap_or(1),
            "favor_point": c.favor_point.unwrap_or(0),
            "skin_id": c.skin.clone().unwrap_or_default(),
            "default_skill": c.default_skill_index.unwrap_or(0),
            "current_equip": c.current_equip.clone(),
            "obtained_at": c.gain_time.unwrap_or(0),
            "voice_lan": c.voice_lan.clone(),
            "current_tmpl": c.current_tmpl.clone(),
        }));
    }

    serde_json::to_value(ops).unwrap_or_default()
}

fn extract_skills(troop: &Option<Troop>) -> serde_json::Value {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return serde_json::json!([]);
    };

    let mut skills: Vec<serde_json::Value> = Vec::new();
    for raw in chars.values() {
        let Ok(c) = serde_json::from_value::<TroopChar>(raw.clone()) else {
            continue;
        };
        let Some(char_id) = c.char_id.clone() else {
            continue;
        };

        if let Some(tmpl) = c.tmpl.as_ref()
            && !tmpl.is_empty()
        {
            for (form_id, form) in tmpl {
                let Some(form_skills) = form.skills.as_ref() else {
                    continue;
                };
                for (i, skill) in form_skills.iter().enumerate() {
                    skills.push(serde_json::json!({
                        "operator_id": form_id,
                        "skill_index": i,
                        "specialize_level": skill.specialize_level.unwrap_or(0),
                    }));
                }
            }
            continue;
        }

        let Some(c_skills) = c.skills else { continue };
        for (i, skill) in c_skills.into_iter().enumerate() {
            skills.push(serde_json::json!({
                "operator_id": char_id,
                "skill_index": i,
                "specialize_level": skill.specialize_level.unwrap_or(0),
            }));
        }
    }

    serde_json::to_value(skills).unwrap_or_default()
}

fn extract_modules(troop: &Option<Troop>) -> serde_json::Value {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return serde_json::json!([]);
    };

    let mut modules: Vec<serde_json::Value> = Vec::new();
    for raw in chars.values() {
        let Ok(c) = serde_json::from_value::<TroopChar>(raw.clone()) else {
            continue;
        };
        let Some(char_id) = c.char_id.clone() else {
            continue;
        };

        if let Some(tmpl) = c.tmpl.as_ref()
            && !tmpl.is_empty()
        {
            for (form_id, form) in tmpl {
                let Some(form_equip) = form.equip.as_ref() else {
                    continue;
                };
                for (module_id, data) in form_equip {
                    if module_id.starts_with("uniequip_001_") {
                        continue;
                    }
                    let Ok(entry) = serde_json::from_value::<EquipEntry>(data.clone()) else {
                        continue;
                    };
                    modules.push(serde_json::json!({
                        "operator_id": form_id,
                        "module_id": module_id,
                        "module_level": entry.level.unwrap_or(0),
                        "locked": entry.locked.map(|l| l != 0).unwrap_or(false),
                    }));
                }
            }
            continue;
        }

        let Some(c_equip) = c.equip else { continue };
        for (module_id, data) in c_equip {
            if module_id.starts_with("uniequip_001_") {
                continue;
            }
            let Ok(entry) = serde_json::from_value::<EquipEntry>(data) else {
                continue;
            };
            modules.push(serde_json::json!({
                "operator_id": char_id,
                "module_id": module_id,
                "module_level": entry.level.unwrap_or(0),
                "locked": entry.locked.map(|l| l != 0).unwrap_or(false),
            }));
        }
    }

    serde_json::to_value(modules).unwrap_or_default()
}

fn extract_items(
    inventory: &Option<serde_json::Map<String, serde_json::Value>>,
) -> serde_json::Value {
    let Some(inv) = inventory.as_ref() else {
        return serde_json::json!([]);
    };

    let items: Vec<serde_json::Value> = inv
        .iter()
        .map(|(id, qty)| {
            serde_json::json!({
                "item_id": id,
                "quantity": qty.as_i64().unwrap_or(0),
            })
        })
        .collect();

    serde_json::to_value(items).unwrap_or_default()
}

fn extract_skins(skin: &Option<SkinStore>) -> serde_json::Value {
    let Some(skins) = skin.as_ref().and_then(|s| s.character_skins.as_ref()) else {
        return serde_json::json!([]);
    };

    let entries: Vec<serde_json::Value> = skins
        .iter()
        .map(|(id, data)| {
            let entry: SkinEntry =
                serde_json::from_value(data.clone()).unwrap_or(SkinEntry { obtained_at: None });
            serde_json::json!({
                "skin_id": id,
                "obtained_at": entry.obtained_at.unwrap_or(0),
            })
        })
        .collect();

    serde_json::to_value(entries).unwrap_or_default()
}

fn extract_status(status: Option<&PlayerStatus>) -> serde_json::Value {
    let Some(s) = status else {
        return serde_json::json!({});
    };

    serde_json::json!({
        "exp": s.exp.unwrap_or(0),
        "orundum": s.orundum.unwrap_or(0),
        "orundum_shard": 0,
        "lmd": s.gold.unwrap_or(0),
        "sanity": s.ap.unwrap_or(0),
        "max_sanity": s.max_ap.unwrap_or(0),
        "gacha_tickets": s.gacha_ticket.unwrap_or(0),
        "ten_pull_tickets": s.ten_gacha_ticket.unwrap_or(0),
        "classic_gacha_tickets": s.classic_gacha_ticket.unwrap_or(0),
        "classic_ten_pull_tickets": s.classic_ten_gacha_ticket.unwrap_or(0),
        "recruit_permits": s.recruit_license.unwrap_or(0),
        "social_point": s.social_point.unwrap_or(0),
        "hgg_shard": s.hgg_shard.unwrap_or(0),
        "lgg_shard": s.lgg_shard.unwrap_or(0),
        "practice_tickets": s.practice_ticket.unwrap_or(0),
        "gold": s.gold.unwrap_or(0),
        "monthly_sub_end": s.monthly_sub_end.unwrap_or(0),
        "register_ts": s.register_ts.unwrap_or(0),
        "last_online_ts": s.last_online_ts.unwrap_or(0),
        "main_stage_progress": s.main_stage_progress.as_deref().unwrap_or(""),
        "resume": s.resume.as_deref().unwrap_or(""),
        "friend_num_limit": s.friend_num_limit.unwrap_or(0),
    })
}

fn extract_medals(medal: &Option<MedalStore>) -> serde_json::Value {
    let Some(medals) = medal.as_ref().and_then(|m| m.medals.as_ref()) else {
        return serde_json::json!([]);
    };

    let entries: Vec<serde_json::Value> = medals
        .iter()
        .filter_map(|(id, data)| {
            // Field-by-field so a non-int `val` (multi-condition medals are
            // arrays like [[achieved, required], ...]) doesn't blow up parsing.
            let val = data.get("val").cloned().unwrap_or(serde_json::Value::Null);
            let fts = data.get("fts").and_then(|v| v.as_i64()).unwrap_or(0);
            let rts = data.get("rts").and_then(|v| v.as_i64()).unwrap_or(0);

            if !is_medal_earned(&val, fts, rts) {
                return None;
            }

            Some(serde_json::json!({
                "medal_id": id,
                "val": val,
                "first_ts": fts,
                "reach_ts": rts,
            }))
        })
        .collect();

    serde_json::to_value(entries).unwrap_or_default()
}

/// Returns true when a medal entry from Hypergryph's `user.medal.medals` map
/// represents an actually earned medal, not in-progress tracking.
///
/// The map includes every medal the account has *seen* (so `fts` is set when
/// the medal becomes tracked, well before completion). Two earn signals:
///   1. `rts > 0` — medal was claimed/awarded; `rts` is the reach timestamp.
///   2. `rts == -1` with `val` showing every `[achieved, required]` pair met —
///      common for story-unlock and one-shot medals that report completion
///      via `val` but never set an explicit reach timestamp.
///
/// Everything else (legacy `rts=0/fts=0` rows from the v2 import, in-progress
/// tracking with partial val, no val at all) is treated as unearned.
pub(crate) fn is_medal_earned(val: &serde_json::Value, fts: i64, rts: i64) -> bool {
    if rts > 0 {
        return true;
    }
    if rts == -1
        && fts > 0
        && let Some(arr) = val.as_array()
        && !arr.is_empty()
        && arr.iter().all(is_condition_met)
    {
        return true;
    }
    false
}

fn is_condition_met(cond: &serde_json::Value) -> bool {
    cond.as_array()
        .and_then(|pair| {
            let a = pair.first()?.as_i64()?;
            let r = pair.get(1)?.as_i64()?;
            Some(a >= r)
        })
        .unwrap_or(false)
}

fn extract_supports(troop: &Option<Troop>, social: &Option<Social>) -> serde_json::Value {
    let Some(list) = social.as_ref().and_then(|s| s.assist_char_list.as_ref()) else {
        return serde_json::json!([]);
    };
    let chars = troop.as_ref().and_then(|t| t.chars.as_ref());

    let entries: Vec<serde_json::Value> = list
        .iter()
        .enumerate()
        .filter_map(|(slot, maybe)| {
            let assist = maybe.as_ref()?;
            let inst_id = assist.char_inst_id?;
            let troop_entry = chars?.get(&inst_id.to_string())?;
            let parsed: TroopChar = serde_json::from_value(troop_entry.clone()).ok()?;
            let char_id = parsed.char_id?;
            let skin_id = parsed.skin.filter(|s| !s.is_empty());
            Some(serde_json::json!({
                "slot": slot as i16,
                "operator_id": char_id,
                "skin_id": skin_id,
                "skill_index": assist.skill_index.unwrap_or(0),
                "current_equip": assist.current_equip,
            }))
        })
        .collect();

    serde_json::to_value(entries).unwrap_or_default()
}

fn extract_roguelike(rlv2: &Option<serde_json::Value>) -> serde_json::Value {
    let Some(outer) = rlv2
        .as_ref()
        .and_then(|v| v.get("outer"))
        .and_then(|o| o.as_object())
    else {
        return serde_json::json!([]);
    };

    let entries: Vec<serde_json::Value> = outer
        .iter()
        .map(|(theme_id, progress)| {
            serde_json::json!({
                "theme_id": theme_id,
                "progress": progress,
            })
        })
        .collect();

    serde_json::to_value(entries).unwrap_or_default()
}
