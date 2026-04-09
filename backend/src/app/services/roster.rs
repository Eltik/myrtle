use chrono::Utc;
use serde::Deserialize;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::{
        grade::calculate::calculate_user_grade,
        hypergryph::{
            constants::{AuthSession, Server},
            fetch::auth_request,
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
    #[serde(rename = "checkIn")]
    pub checkin: Option<CheckIn>,
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
    pub skills: Option<Vec<TroopSkill>>,
    pub equip: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TroopSkill {
    pub specialize_level: Option<i64>,
}

#[derive(Deserialize)]
pub struct EquipEntry {
    pub level: Option<i64>,
    pub locked: Option<bool>,
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
pub struct MedalEntry {
    pub val: Option<i64>,
    pub fts: Option<i64>,
    pub rts: Option<i64>,
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

    let response = auth_request(
        &state.http_client,
        "account/syncData",
        Some(&serde_json::json!({"platform": 1})),
        &mut session,
        server,
    )
    .await?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let data: SyncDataResponse =
        serde_json::from_slice(&bytes).map_err(|e| ApiError::Internal(e.into()))?;

    let raw: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| ApiError::Internal(e.into()))?;

    let user = data
        .user
        .ok_or(ApiError::BadRequest("missing user data in response".into()))?;

    let status = user.status.as_ref();

    let nickname = status.and_then(|s| s.nick_name.as_deref()).unwrap_or("");
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
    let stages = user
        .dungeon
        .as_ref()
        .and_then(|d| d.get("stages"))
        .cloned()
        .unwrap_or_default();
    let roguelike = extract_roguelike(&user.roguelike);
    let sandbox = user.sandbox.unwrap_or_default();
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
    )
    .await?;

    if let Some(user) = users::find_by_uid(&state.db, user_id).await? {
        let grade = calculate_user_grade(&state.db, user.id, &state.game_data).await?;

        score::update_score(
            &state.db,
            &UserScore {
                user_id: user.id,
                operator_score: grade.operator_grade,
                total_score: grade.total_score, // for now, just operator
                grade: Some(grade.overall),
                // zero out the rest until implemented
                stage_score: 0.0,
                roguelike_score: 0.0,
                sandbox_score: 0.0,
                medal_score: 0.0,
                base_score: grade.base_grade,
                skin_score: 0.0,
                calculated_at: Utc::now(),
            },
        )
        .await?;
    }

    Ok(raw)
}

fn extract_operators(troop: &Option<Troop>) -> serde_json::Value {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return serde_json::json!([]);
    };

    let ops: Vec<serde_json::Value> = chars
        .values()
        .filter_map(|raw| {
            let c: TroopChar = serde_json::from_value(raw.clone()).ok()?;
            let char_id = c.char_id?;
            Some(serde_json::json!({
                "operator_id": char_id,
                "elite": c.evolve_phase.unwrap_or(0),
                "level": c.level.unwrap_or(1),
                "exp": c.exp.unwrap_or(0),
                "potential": c.potential_rank.unwrap_or(0),
                "skill_level": c.main_skill_lvl.unwrap_or(1),
                "favor_point": c.favor_point.unwrap_or(0),
                "skin_id": c.skin.unwrap_or_default(),
                "default_skill": c.default_skill_index.unwrap_or(0),
                "current_equip": c.current_equip,
                "obtained_at": c.gain_time.unwrap_or(0),
            }))
        })
        .collect();

    serde_json::to_value(ops).unwrap_or_default()
}

fn extract_skills(troop: &Option<Troop>) -> serde_json::Value {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return serde_json::json!([]);
    };

    let skills: Vec<serde_json::Value> = chars
        .values()
        .flat_map(|raw| {
            let c: TroopChar = serde_json::from_value(raw.clone()).ok()?;
            let char_id = c.char_id?;
            Some(
                c.skills
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                    .map(move |(i, skill)| {
                        serde_json::json!({
                            "operator_id": char_id,
                            "skill_index": i,
                            "specialize_level": skill.specialize_level.unwrap_or(0),
                        })
                    }),
            )
        })
        .flatten()
        .collect();

    serde_json::to_value(skills).unwrap_or_default()
}

fn extract_modules(troop: &Option<Troop>) -> serde_json::Value {
    let Some(chars) = troop.as_ref().and_then(|t| t.chars.as_ref()) else {
        return serde_json::json!([]);
    };

    let modules: Vec<serde_json::Value> =
        chars
            .values()
            .flat_map(|raw| {
                let c: TroopChar = serde_json::from_value(raw.clone()).ok()?;
                let char_id = c.char_id?;
                Some(c.equip.unwrap_or_default().into_iter().filter_map(
                    move |(module_id, data)| {
                        if module_id.starts_with("uniequip_001_") {
                            return None;
                        }
                        let entry: EquipEntry = serde_json::from_value(data).ok()?;
                        Some(serde_json::json!({
                            "operator_id": char_id,
                            "module_id": module_id,
                            "module_level": entry.level.unwrap_or(0),
                            "locked": entry.locked.unwrap_or(false),
                        }))
                    },
                ))
            })
            .flatten()
            .collect();

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
        .map(|(id, data)| {
            let entry: MedalEntry = serde_json::from_value(data.clone()).unwrap_or(MedalEntry {
                val: None,
                fts: None,
                rts: None,
            });
            serde_json::json!({
                "medal_id": id,
                "val": entry.val.unwrap_or(0),
                "first_ts": entry.fts.unwrap_or(0),
                "reach_ts": entry.rts.unwrap_or(0),
            })
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
