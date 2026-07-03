use chrono::Utc;
use serde::Deserialize;

use crate::database::queries::roster::sync_user_data;
use crate::database::queries::score::update_score;
use crate::database::queries::users::find_by_uid;
use crate::{
    app::{error::ApiError, services::game_session, state::AppState},
    core::{
        gamedata::types::campaign::CampaignRotations,
        grade::calculate::calculate_user_grade,
        hypergryph::{constants::Server, yostar::sync_data_raw},
    },
    database::models::score::UserScore,
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
    /// Up to 3 entries (some may be `null` for empty slots) - references
    /// troop slots by `charInstId`. Resolved to `operator_id` in
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
    /// Current month's daily sign-in calendar (0/1 per day). The raw key is
    /// `checkInHistory`; the bare `history` key does not exist on this object.
    #[serde(rename = "checkInHistory")]
    pub history: Option<Vec<i16>>,
    /// Lifetime cumulative sign-in days — the "X / 1000 total days of sign-ins"
    /// counter shown on the daily sign-in carousel's milestone page. Distinct
    /// from `history`, which is only the current month's calendar.
    #[serde(rename = "showCount")]
    pub show_count: Option<i64>,
    /// Identifier of the active monthly sign-in series (e.g. `signin<N>`).
    #[serde(rename = "checkInGroupId")]
    pub group_id: Option<String>,
    /// Days already claimed in the current month's calendar.
    #[serde(rename = "checkInRewardIndex")]
    pub reward_index: Option<i64>,
    /// Whether a daily sign-in is claimable right now (0/1, as of this sync).
    #[serde(rename = "canCheckIn")]
    pub can_check_in: Option<i64>,
}

pub async fn refresh(
    state: &AppState,
    user_id: &str,
    server: Server,
) -> Result<serde_json::Value, ApiError> {
    let mut session = game_session::ensure_fresh(state, user_id, server).await?;

    let text = match sync_data_raw(&state.http_client, &mut session, server).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(
                uid = %user_id,
                server = server.as_str(),
                error = ?e,
                "account/syncData failed"
            );
            return Err(e.into());
        }
    };

    game_session::save(state, user_id, &session).await;

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
    let mut stages = user
        .dungeon
        .as_ref()
        .and_then(|d| d.get("stages"))
        .cloned()
        .unwrap_or_default();
    merge_campaign_clears(
        &mut stages,
        &raw,
        &state.default_game_data().campaign_rotations,
    );
    let roguelike = extract_roguelike(&user.roguelike);
    let sandbox = user.sandbox_perm.unwrap_or_default();
    let medals = extract_medals(&user.medal);
    let building = user.building.unwrap_or_default();
    let checkin = extract_checkin(&user.checkin);
    let enemies = raw
        .pointer("/user/dexNav/enemy/enemies")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    sync_user_data(
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
        &enemies,
    )
    .await?;

    if let Some(user) = find_by_uid(&state.db, user_id).await? {
        let grade = calculate_user_grade(&state.db, user.id, &state.default_game_data()).await?;

        update_score(
            &state.db,
            &UserScore {
                user_id: user.id,
                operator_score: grade.operator_grade,
                total_score: grade.total_score, // operator grade only
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

/// Parse the troop map into `(char_id, TroopChar)` pairs, skipping entries
/// that fail to parse or lack a char id.
fn troop_chars(troop: &Option<Troop>) -> Vec<(String, TroopChar)> {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return Vec::new();
    };
    chars
        .values()
        .filter_map(|raw| {
            let c: TroopChar = serde_json::from_value(raw.clone()).ok()?;
            let char_id = c.char_id.clone()?;
            Some((char_id, c))
        })
        .collect()
}

fn extract_operators(troop: &Option<Troop>) -> serde_json::Value {
    let mut ops: Vec<serde_json::Value> = Vec::new();
    for (char_id, c) in troop_chars(troop) {
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
    let mut skills: Vec<serde_json::Value> = Vec::new();
    for (char_id, c) in troop_chars(troop) {
        if let Some(tmpl) = c.tmpl.as_ref()
            && !tmpl.is_empty()
        {
            for (form_id, form) in tmpl {
                if let Some(form_skills) = form.skills.as_ref() {
                    push_skills(&mut skills, form_id, form_skills);
                }
            }
            continue;
        }

        if let Some(c_skills) = c.skills.as_ref() {
            push_skills(&mut skills, &char_id, c_skills);
        }
    }

    serde_json::to_value(skills).unwrap_or_default()
}

fn push_skills(out: &mut Vec<serde_json::Value>, operator_id: &str, skills: &[TroopSkill]) {
    for (i, skill) in skills.iter().enumerate() {
        out.push(serde_json::json!({
            "operator_id": operator_id,
            "skill_index": i,
            "specialize_level": skill.specialize_level.unwrap_or(0),
        }));
    }
}

fn extract_modules(troop: &Option<Troop>) -> serde_json::Value {
    let mut modules: Vec<serde_json::Value> = Vec::new();
    for (char_id, c) in troop_chars(troop) {
        if let Some(tmpl) = c.tmpl.as_ref()
            && !tmpl.is_empty()
        {
            for (form_id, form) in tmpl {
                if let Some(form_equip) = form.equip.as_ref() {
                    push_modules(&mut modules, form_id, form_equip);
                }
            }
            continue;
        }

        if let Some(c_equip) = c.equip.as_ref() {
            push_modules(&mut modules, &char_id, c_equip);
        }
    }

    serde_json::to_value(modules).unwrap_or_default()
}

fn push_modules(
    out: &mut Vec<serde_json::Value>,
    operator_id: &str,
    equip: &serde_json::Map<String, serde_json::Value>,
) {
    for (module_id, data) in equip {
        // uniequip_001_* is the default badge every operator owns, not a real module.
        if module_id.starts_with("uniequip_001_") {
            continue;
        }
        let Ok(entry) = serde_json::from_value::<EquipEntry>(data.clone()) else {
            continue;
        };
        out.push(serde_json::json!({
            "operator_id": operator_id,
            "module_id": module_id,
            "module_level": entry.level.unwrap_or(0),
            "locked": entry.locked.is_some_and(|l| l != 0),
        }));
    }
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

/// Flatten the `checkIn` section into the JSONB payload `sp_sync_user_data`
/// stores in `user_checkin`: the current month's calendar plus the lifetime
/// sign-in counter and the active monthly series' progress.
fn extract_checkin(checkin: &Option<CheckIn>) -> serde_json::Value {
    let Some(c) = checkin else {
        return serde_json::json!({ "history": [] });
    };

    serde_json::json!({
        "history": c.history.clone().unwrap_or_default(),
        "cumulative_signin": c.show_count.unwrap_or(0),
        "group_id": c.group_id,
        "reward_index": c.reward_index.unwrap_or(0),
        "can_check_in": c.can_check_in.is_some_and(|v| v != 0),
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
            let fts = data
                .get("fts")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            let rts = data
                .get("rts")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);

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
/// Hypergryph's unearned-default row is `{val: 0, fts: 0, rts: 0}` - those are
/// initialized but never touched by gameplay. Earn signals:
///   1. `rts > 0` - medal was claimed/awarded; `rts` is the reach timestamp.
///   2. `rts == -1` with `fts > 0` and `val` being an array:
///      - Non-empty: every `[achieved, required]` pair must be met (story
///        unlocks, multi-step medals).
///      - Empty `[]`: "binary completion" templates with no trackable
///        condition pairs (`PassStageKilled`, `PassStageWithSimpleCount*`,
///        etc.). For these, Hypergryph stamps `fts` + `rts=-1` on award and
///        leaves `val` at its zero-condition shape; that combo is the earn
///        signal. Empirically, ~30% of activity-medal templates use this
///        pattern (see Break the Ice medals 01/02/07/09/10).
///
/// Everything else (rts=0/fts=0 defaults, in-progress with partial val, no
/// val at all) is treated as unearned.
pub(crate) fn is_medal_earned(val: &serde_json::Value, fts: i64, rts: i64) -> bool {
    if rts > 0 {
        return true;
    }
    if rts == -1
        && fts > 0
        && let Some(arr) = val.as_array()
    {
        if arr.is_empty() {
            return true;
        }
        return arr.iter().all(is_condition_met);
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

/// Annihilation progress lives at `user.campaignsV2.instances` (top level of
/// the save, not under `user.dungeon`) and is kill-count based - there is no
/// `state` flag like normal stages. Merge it into the stages object with a
/// synthesized `state` (kill target reached → 3, partial kills → 2) so every
/// downstream stage-clear consumer treats Annihilation like any other stage.
fn merge_campaign_clears(
    stages: &mut serde_json::Value,
    raw: &serde_json::Value,
    campaign: &CampaignRotations,
) {
    let Some(instances) = raw
        .pointer("/user/campaignsV2/instances")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    if instances.is_empty() {
        return;
    }

    if !stages.is_object() {
        *stages = serde_json::json!({});
    }
    let Some(map) = stages.as_object_mut() else {
        return;
    };

    for (stage_id, instance) in instances {
        if map.contains_key(stage_id) {
            continue;
        }
        let kills = instance
            .get("maxKills")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        if kills <= 0 {
            continue;
        }
        let full = campaign
            .kill_max(stage_id)
            .is_some_and(|max| kills >= i64::from(max));
        map.insert(
            stage_id.clone(),
            serde_json::json!({
                "stageId": stage_id,
                "state": if full { 3 } else { 2 },
                "completeTimes": 1,
                "practiceTimes": 0,
                "campaignMaxKills": kills,
            }),
        );
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::campaign::CampaignTableFile;

    fn rotations() -> CampaignRotations {
        let table: CampaignTableFile = serde_json::from_value(serde_json::json!({
            "Campaigns": [
                {"key": "camp_01", "value": {"BreakLadders": [{"KillCnt": 100}, {"KillCnt": 400}]}},
                {"key": "camp_r_01", "value": {"BreakLadders": [{"KillCnt": 400}]}}
            ]
        }))
        .unwrap();
        CampaignRotations::from_table(table)
    }

    #[test]
    fn campaign_clears_merge_into_stages() {
        let raw = serde_json::json!({
            "user": {
                "campaignsV2": {
                    "instances": {
                        "camp_01": {"maxKills": 400, "rewardStatus": [1,1,1,1,1,1,1,1]},
                        "camp_r_01": {"maxKills": 250, "rewardStatus": [1,1,1,0,0,0,0,0]},
                        "camp_r_02": {"maxKills": 0, "rewardStatus": [0,0,0,0,0,0,0,0]}
                    }
                }
            }
        });
        let mut stages = serde_json::json!({
            "main_00-01": {"stageId": "main_00-01", "state": 3, "completeTimes": 1, "practiceTimes": 0}
        });

        merge_campaign_clears(&mut stages, &raw, &rotations());

        // Kill target reached -> full clear.
        assert_eq!(stages["camp_01"]["state"], 3);
        assert_eq!(stages["camp_01"]["campaignMaxKills"], 400);
        // Partial kills -> cleared but not maxed.
        assert_eq!(stages["camp_r_01"]["state"], 2);
        // Never played -> no synthesized entry.
        assert!(stages.get("camp_r_02").is_none());
        // Existing stage entries untouched.
        assert_eq!(stages["main_00-01"]["state"], 3);
    }

    #[test]
    fn campaign_merge_handles_missing_stages_and_unknown_maps() {
        // dungeon.stages absent -> stages defaults to Null; the merge must
        // still produce an object.
        let raw = serde_json::json!({
            "user": {"campaignsV2": {"instances": {
                // Not in the campaign table (no kill_max) - partial credit only.
                "camp_r_99": {"maxKills": 400}
            }}}
        });
        let mut stages = serde_json::Value::Null;

        merge_campaign_clears(&mut stages, &raw, &rotations());

        assert_eq!(stages["camp_r_99"]["state"], 2);
    }

    #[test]
    fn campaign_merge_prefers_existing_dungeon_entry() {
        let raw = serde_json::json!({
            "user": {"campaignsV2": {"instances": {
                "camp_01": {"maxKills": 400}
            }}}
        });
        let mut stages = serde_json::json!({
            "camp_01": {"stageId": "camp_01", "state": 2, "completeTimes": 5, "practiceTimes": 0}
        });

        merge_campaign_clears(&mut stages, &raw, &rotations());

        // A real save-provided record wins over the synthesized one.
        assert_eq!(stages["camp_01"]["completeTimes"], 5);
        assert_eq!(stages["camp_01"]["state"], 2);
    }
}
