use chrono::Utc;
use serde::Deserialize;

use crate::database::queries::roster::sync_user_data;
use crate::database::queries::score::update_score;
use crate::database::queries::users::find_by_uid;
use crate::{
    app::{error::ApiError, services::game_session, state::AppState},
    core::{
        gamedata::types::{
            campaign::CampaignRotations,
            stage_evidence::{self, StageEvidenceIndex},
        },
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
    /// Vouchers, selectors and packs: `itemId -> instId -> { ts, count }`,
    /// one instance per grant, `ts` its expiry (-1 = never). Not part of
    /// `inventory`, so it has to be read on its own.
    pub consumable: Option<serde_json::Map<String, serde_json::Value>>,
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
    /// Expedited Plans (item 7002) and Universal Certificates (item
    /// `classic_normal_ticket`): `status` fields like the other tickets, but
    /// stored as `user_items` rows so they need no column.
    pub instant_finish_ticket: Option<i64>,
    pub classic_shard: Option<i64>,
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
    /// One flag per sign-in **claimed** this month, in claim order: `1` if the
    /// monthly-subscription Daily Supply came with that claim, `0` if not.
    /// Despite the name it is not a dated calendar: a missed day adds no
    /// entry, so its length is the claim count and its values say nothing
    /// about which days were claimed. The raw key is `checkInHistory`; the
    /// bare `history` key does not exist on this object.
    #[serde(rename = "checkInHistory")]
    pub history: Option<Vec<i16>>,
    /// Lifetime cumulative sign-in days, the "X / 1000 total days of sign-ins"
    /// counter shown on the daily sign-in carousel's milestone page. Distinct
    /// from `history`, which is only the current month's calendar.
    #[serde(rename = "showCount")]
    pub show_count: Option<i64>,
    /// Identifier of the active monthly sign-in series (e.g. `signin<N>`).
    #[serde(rename = "checkInGroupId")]
    pub group_id: Option<String>,
    /// 0-based pointer into the month's reward slots: the claim count while a
    /// claim is pending, one less just after one is taken.
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
    dump_sync_data(user_id, server, &text);

    let data: SyncDataResponse =
        serde_json::from_str(&text).map_err(|e| ApiError::Internal(e.into()))?;

    let raw: std::sync::Arc<serde_json::Value> =
        std::sync::Arc::new(serde_json::from_str(&text).map_err(|e| ApiError::Internal(e.into()))?);

    let game_story = game_story_read(state, user_id, server, &raw).await;
    log_story_block(user_id, &raw, &game_story);

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
    let items = extract_items(
        &user.inventory,
        &user.consumable,
        &[
            (
                EXPEDITED_PLAN_ITEM,
                status.and_then(|s| s.instant_finish_ticket),
            ),
            (
                UNIVERSAL_CERTIFICATE_ITEM,
                status.and_then(|s| s.classic_shard),
            ),
        ],
    );
    let skins = extract_skins(&user.skin);
    let mut status_json = extract_status(status);
    status_json["originite"] =
        extract_originite(&raw).map_or(serde_json::Value::Null, |op| serde_json::json!(op));
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
    merge_inferred_clears(&mut stages, &raw, &state.game_data(server).stage_evidence);
    merge_cow_level(&mut stages, &raw);
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

    // What the game says has been read goes in BEFORE the grade, and the grade
    // goes off the async runtime. Both halves of that are the 2026-09-24
    // defect.
    // `calculate_user_grade` scores INLINE on this task: 40,327 ms standalone in
    // a debug build, 73,011 ms of the failing request's 75.967 s as
    // `myrtle_cpu_task_duration_seconds_sum{kind="user_grade"}` measured it. A
    // task inside inline compute is never polled, so the 30 s handler timeout in
    // `middleware` could not be OBSERVED until the scoring returned, and
    // `tokio::time::timeout` polls its inner future first, so it fired at the
    // first await afterwards and DROPPED everything behind it. That is why the
    // request answered 5xx at 75.967 s, `user_scores` was written (its INSERT
    // had been sent) and `user_game_story_read` held 0 rows. The store itself is
    // 19 ms for 1,365 rows, so it belongs ahead of the scoring, not behind it.
    let mut import = serde_json::json!({
        "ok": false,
        "error": "the account row was not found after the sync",
    });

    if let Some(user) = find_by_uid(&state.db, user_id).await? {
        import = import_game_read(state, user_id, user.id, &game_story).await;

        // The grade on its own task, bounded, and the scoring on the BLOCKING
        // POOL rather than on a runtime worker.
        //
        // Its own task was not enough. Measured 2026-09-24: the grade held its
        // admission for 73.620 s
        // (`myrtle_cpu_task_duration_seconds_sum{kind="user_grade"}`), and this
        // 12 s budget was not observed until 23:36:36.532, 73.6 s after the
        // import at 23:35:22.916, so the deadline passed 61.6 s unnoticed. A
        // `tokio::time::timeout` is only observed when its own task is polled,
        // and with one worker inside a non-yielding 73 s compute the runtime's
        // timer was not serviced on time either. `cpu::admit` bounded how many
        // such computes could run at once; it never took the compute off the
        // worker, which is exactly the distinction `cpu`'s module doc draws.
        // Measured offline on the same account, one runtime, the same probe
        // running `/auth/verify`'s two queries every 100 ms
        // (`sync_stall_test::the_grade_budget_is_observed_on_time_only_off_the_runtime`):
        // on a worker the budget was observed at 39.604 s and 43.156 s over two
        // runs, both of them the instant the grade RETURNED (39.603 s,
        // 43.156 s); off it, 12.003 s and 12.002 s while the grade still took
        // 40.373 s and 40.908 s. The session probe was never the casualty: its
        // worst was 0.012 s and 0.021 s on a worker, so nothing in this path
        // held a lock a user route needed.
        //
        // `calculate_user_grade` is an `async fn` whose only awaits are the
        // seven up-front queries, and `core/grade` is another session's file,
        // so it is not split here into load-then-score. Instead the whole call
        // is DRIVEN from a blocking-pool thread: `Handle::block_on` inside
        // `cpu::run` parks that thread on the queries, which stay registered
        // with this runtime's IO driver, and runs the scoring there. No async
        // worker is held at any point, so the awaited `JoinHandle` yields and
        // the budget below fires at 12 s.
        let db = state.db.clone();
        let scoring_db = state.db.clone();
        let game_data = state.default_game_data();
        let graded = user.id;
        let handle = tokio::runtime::Handle::current();
        let grading = tokio::spawn(async move {
            let grade = crate::app::cpu::run("user_grade", move || {
                handle.block_on(calculate_user_grade(&scoring_db, graded, &game_data))
            })
            .await??;
            update_score(
                &db,
                &UserScore {
                    user_id: graded,
                    operator_score: grade.operator_grade,
                    total_score: grade.total_score, // operator grade only
                    grade: Some(grade.overall),
                    stage_score: grade.stage_grade,
                    roguelike_score: grade.roguelike_grade,
                    sandbox_score: grade.sandbox_grade,
                    medal_score: grade.medal_grade,
                    base_score: grade.base_grade,
                    base_utilization: Some(grade.base_utilization),
                    base_infrastructure: Some(grade.base_infrastructure),
                    skin_score: 0.0,
                    calculated_at: Utc::now(),
                },
            )
            .await?;
            Ok::<(), ApiError>(())
        });
        match tokio::time::timeout(GRADE_BUDGET, grading).await {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(e))) => {
                tracing::error!(uid = %user_id, error = ?e, "the grade failed; the roster sync stands");
            }
            Ok(Err(e)) => {
                tracing::error!(uid = %user_id, error = %e, "the grade task panicked; the roster sync stands");
            }
            Err(_) => tracing::error!(
                uid = %user_id,
                budget_secs = GRADE_BUDGET.as_secs(),
                "the grade did not finish inside the budget; it continues in the background and user_scores updates when it does"
            ),
        }
    }

    state.mark_ownership_dirty();
    state
        .cache
        .invalidate(&crate::app::cache::keys::CacheKey::User { uid: user_id })
        .await;
    // Improvements bodies are keyed on this user's sync generation, so the sync
    // that just happened already lands on a new key and cannot serve a stale
    // body. This clears the superseded ones rather than leaving them to age out,
    // which matters because they run to hundreds of kilobytes each.
    state
        .cache
        .invalidate_by_prefix(&format!("improvements:{user_id}:"))
        .await;

    // The import rides back on the payload the route passes through, so the
    // Progress tab can say "imported" or say why not, instead of the refresh
    // silently answering 200 with nothing stored.
    let mut payload = std::sync::Arc::try_unwrap(raw).unwrap_or_else(|shared| (*shared).clone());
    if let Some(object) = payload.as_object_mut() {
        object.insert("gameReadImport".to_owned(), import);
    }
    Ok(payload)
}

/// How long the story index may take before the flags half of the game's
/// reading record is given up on for this refresh.
const STORY_INDEX_BUDGET: std::time::Duration = std::time::Duration::from_secs(30);

/// How long the game-read store may take before the refresh abandons it.
const GAME_READ_STORE_BUDGET: std::time::Duration = std::time::Duration::from_secs(10);

/// How long the refresh waits for the grade before it answers without it.
///
/// The handler timeout in `middleware` is 30 s and everything before the grade
/// has to fit inside it too, so this is deliberately well under it: a grade
/// that overruns keeps running on its own task and writes `user_scores` when
/// it finishes, and the refresh that triggered it answers with the roster the
/// user asked for.
///
/// This is only a budget when the grade is off the async runtime. While the
/// scoring ran on a worker it was observed at 73.620 s instead of 12 s
/// (2026-09-24); see the comment at the spawn.
const GRADE_BUDGET: std::time::Duration = std::time::Duration::from_secs(12);

/// Store the game's own read marks, and say what happened in a form the
/// Progress tab can show.
///
/// Non-fatal on every axis: the roster sync has already COMMITTED by the time
/// this runs, so nothing here may cost the user their data. A failure is loud
/// (`error!`, with the stage and the elapsed ms) and travels back as
/// `gameReadImport.ok = false` rather than as a 5xx.
async fn import_game_read(
    state: &AppState,
    uid: &str,
    user_id: uuid::Uuid,
    set: &crate::app::services::story_progress::GameStoryReadSet,
) -> serde_json::Value {
    let started = std::time::Instant::now();
    match tokio::time::timeout(
        GAME_READ_STORE_BUDGET,
        crate::app::services::story_progress::store_game_read(state, user_id, set),
    )
    .await
    {
        Ok(Ok(outcome)) => {
            if set.present {
                tracing::info!(
                    uid,
                    rows = outcome.rows,
                    read = set.read_count(),
                    archived = set.archived(),
                    delete_ms = outcome.delete_ms as u64,
                    insert_ms = outcome.insert_ms as u64,
                    commit_ms = outcome.commit_ms as u64,
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "game story read marks imported"
                );
            }
            serde_json::json!({
                "ok": true,
                "rows": outcome.rows,
                "read": set.read_count(),
                "archived": set.archived(),
                "elapsedMs": started.elapsed().as_millis() as u64,
            })
        }
        Ok(Err(e)) => {
            tracing::error!(
                uid,
                stage = e.stage,
                stage_ms = e.elapsed_ms as u64,
                elapsed_ms = started.elapsed().as_millis() as u64,
                error = %e.message,
                "game story read import FAILED; the roster sync stands"
            );
            serde_json::json!({ "ok": false, "error": e.to_string() })
        }
        Err(_) => {
            let message = format!(
                "the game read store did not answer inside {} s",
                GAME_READ_STORE_BUDGET.as_secs()
            );
            tracing::error!(
                uid,
                budget_secs = GAME_READ_STORE_BUDGET.as_secs(),
                rows_expected = set.read.len(),
                "game story read import TIMED OUT; the roster sync stands"
            );
            serde_json::json!({ "ok": false, "error": message })
        }
    }
}

/// Both halves of the game's own reading record, parsed OFF the async worker.
///
/// The flags half is a set of script paths and needs the story index's
/// `StoryTxt` -> id map, so this reaches the index. An index that cannot be
/// built, or does not build inside [`STORY_INDEX_BUDGET`], yields an EMPTY map
/// rather than an error: the Archive half still imports and the census reports
/// `flag_hits` 0, which is how that degradation shows up in the log.
///
/// The parse goes to the blocking pool because it walks a 2.4 MB payload, and,
/// when no named Archive block is found, walks it again to depth 5 for the log
/// line. Neither belongs on a tokio worker. It takes no CPU permit: a census
/// must never be the reason a refresh is shed.
async fn game_story_read(
    state: &AppState,
    uid: &str,
    server: Server,
    raw: &std::sync::Arc<serde_json::Value>,
) -> crate::app::services::story_progress::GameStoryReadSet {
    let index = match tokio::time::timeout(
        STORY_INDEX_BUDGET,
        crate::app::services::story::cached_index(state, server),
    )
    .await
    {
        Ok(Ok(index)) => Some(index),
        Ok(Err(e)) => {
            tracing::warn!(uid, error = ?e, "story index unavailable; the flags half is skipped");
            None
        }
        Err(_) => {
            tracing::warn!(
                uid,
                budget_secs = STORY_INDEX_BUDGET.as_secs(),
                "story index did not build inside the budget; the flags half is skipped"
            );
            None
        }
    };
    let raw = std::sync::Arc::clone(raw);
    crate::app::cpu::offload("sync_story_census", move || {
        let empty = std::collections::HashMap::new();
        let no_gates = std::collections::HashMap::new();
        let by_txt = index.as_ref().map_or(&empty, |i| &i.by_txt);
        let gates = index.as_ref().map_or(&no_gates, |i| &i.gates);
        crate::app::services::story_progress::parse_game_story_read(&raw, by_txt, gates)
    })
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(uid, error = ?e, "story census failed; nothing is imported");
        crate::app::services::story_progress::GameStoryReadSet::default()
    })
}

/// Originite Prime is `status.payDiamond` + `status.freeDiamond`; absent
/// when the payload carries neither.
fn extract_originite(raw: &serde_json::Value) -> Option<i64> {
    let number = |key: &str| {
        raw.pointer(&format!("/user/status/{key}"))
            .and_then(serde_json::Value::as_i64)
    };
    match (number("payDiamond"), number("freeDiamond")) {
        (None, None) => None,
        (pay, free) => Some(pay.unwrap_or(0) + free.unwrap_or(0)),
    }
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

/// Item ids of the counters the game keeps on `status` rather than in
/// `inventory`: Expedited Plans (`instantFinishTicket`) and Universal
/// Certificates (`classicShard`, the `CLASSIC_SHD` item).
const EXPEDITED_PLAN_ITEM: &str = "7002";
const UNIVERSAL_CERTIFICATE_ITEM: &str = "classic_normal_ticket";

/// Every `(item_id, quantity)` the account holds, from the three places the
/// game keeps them: the `inventory` map, the `consumable` block summed per
/// item over its unexpired instances, and the `status` counters passed as
/// `(item_id, count)`. One shape out, so the inventory and the leaderboard
/// see all of them alike.
fn extract_items(
    inventory: &Option<serde_json::Map<String, serde_json::Value>>,
    consumable: &Option<serde_json::Map<String, serde_json::Value>>,
    status_counters: &[(&str, Option<i64>)],
) -> serde_json::Value {
    let mut quantities: std::collections::BTreeMap<&str, i64> = inventory
        .iter()
        .flat_map(|inv| inv.iter())
        .map(|(id, qty)| (id.as_str(), qty.as_i64().unwrap_or(0)))
        .collect();

    let now = chrono::Utc::now().timestamp();
    for (id, instances) in consumable.iter().flat_map(|c| c.iter()) {
        let total: i64 = instances
            .as_object()
            .into_iter()
            .flat_map(|insts| insts.values())
            .filter(|inst| {
                let expiry = inst
                    .get("ts")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(-1);
                expiry < 0 || expiry > now
            })
            .filter_map(|inst| inst.get("count").and_then(serde_json::Value::as_i64))
            .sum();
        if total > 0 {
            *quantities.entry(id.as_str()).or_insert(0) += total;
        }
    }

    for (id, count) in status_counters {
        if let Some(count) = count.filter(|n| *n > 0) {
            *quantities.entry(id).or_insert(0) += count;
        }
    }

    let items: Vec<serde_json::Value> = quantities
        .into_iter()
        .map(|(id, quantity)| serde_json::json!({ "item_id": id, "quantity": quantity }))
        .collect();
    serde_json::to_value(items).unwrap_or_default()
}

/// Write the raw `syncData` text to `$MYRTLE_SYNC_DUMP_DIR/<server>_<uid>_<unix>.json`
/// when that variable is set. Off by default: the payload is the whole account.
/// Exists so a field the extractors do not read yet can be looked up at the
/// source instead of guessed.
fn dump_sync_data(uid: &str, server: Server, text: &str) {
    let Ok(dir) = std::env::var("MYRTLE_SYNC_DUMP_DIR") else {
        return;
    };
    if dir.is_empty() {
        return;
    }
    let path = std::path::Path::new(&dir).join(format!(
        "{}_{uid}_{}.json",
        server.as_str(),
        chrono::Utc::now().timestamp()
    ));
    match std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, text)) {
        Ok(()) => tracing::info!(uid, path = %path.display(), "syncData dumped"),
        Err(e) => tracing::warn!(uid, error = %e, "syncData dump failed"),
    }
}

/// Log what the payload's top level holds and what the two reading sources say.
/// One line per refresh, no user data beyond a single story id and three
/// unmatched script paths.
///
/// The numbers that matter are `flag_hits` (played-script flags that map to a
/// story), `review_hits` (Archive ids the index knows), `stage_hits` (gated
/// stories whose stage record satisfies every gate), `union` (rows stored),
/// `read` (rows the verdict marks read) and `archive_only_unread` (Archive
/// entries on a gated story nothing played). Against the 2026-09-24 EN payload
/// those read 1,084, 1,020, 1,061, 1,407, 1,313 and 94.
fn log_story_block(
    uid: &str,
    raw: &serde_json::Value,
    set: &crate::app::services::story_progress::GameStoryReadSet,
) {
    let user_keys: Vec<&str> = raw
        .get("user")
        .and_then(serde_json::Value::as_object)
        .map(|o| o.keys().map(String::as_str).collect())
        .unwrap_or_default();
    tracing::info!(
        uid,
        ?user_keys,
        story_block = set.present,
        key = ?set.key,
        groups = set.groups,
        stories = set.stories,
        flags = set.flags,
        flag_hits = set.flag_hits,
        flag_misses = set.flag_misses,
        flag_miss_sample = ?set.flag_miss_sample,
        review_hits = set.review_hits,
        stage_records = set.stage_records,
        stage_hits = set.stage_hits,
        archive_only_unread = set.archive_only_unread,
        union = set.read.len(),
        read = set.read_count(),
        sample = ?set.sample,
        "syncData story block census"
    );
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
/// stores in `user_checkin`: the month's per-claim monthly-card flags plus the
/// lifetime sign-in counter and the active monthly series' progress. The
/// `history` key is kept verbatim from the game (see [`CheckIn::history`] for
/// what it does and does not encode). The API layer renames it.
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
/// synthesized `state` (kill target reached -> 3, partial kills -> 2) so every
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

/// The client drops a closed event's battle records; fold in what the
/// account's surviving mission, medal, story-flag and unlock records still
/// prove about those stages (see `stage_evidence`).
/// Copy each `dungeon.cowLevel` first-open time onto its stage record as
/// `cowFirstTs`.
///
/// The special story stages (`spst_*`, choices and all) keep their opening in
/// `cowLevel.<stageId>.fts` and NOWHERE in `dungeon.stages`, whose record sits
/// at state 3 with zero starts and zero completions like every story-only
/// stage. The story verdict needs that opening after the payload is gone (the
/// "Sync now" re-derivation reads the stored records), so it rides on the
/// record under a name the game does not use. Only records that exist are
/// annotated, and only openings that happened (`fts` above zero).
fn merge_cow_level(stages: &mut serde_json::Value, raw: &serde_json::Value) {
    let Some(cow) = raw
        .pointer("/user/dungeon/cowLevel")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    let Some(map) = stages.as_object_mut() else {
        return;
    };
    for (id, entry) in cow {
        let Some(fts) = entry.get("fts").and_then(serde_json::Value::as_i64) else {
            continue;
        };
        if fts <= 0 {
            continue;
        }
        if let Some(record) = map.get_mut(id).and_then(serde_json::Value::as_object_mut) {
            record.insert("cowFirstTs".to_owned(), serde_json::Value::from(fts));
        }
    }
}

fn merge_inferred_clears(
    stages: &mut serde_json::Value,
    raw: &serde_json::Value,
    evidence: &StageEvidenceIndex,
) {
    let Some(user) = raw.get("user") else {
        return;
    };
    let bounds = evidence.infer(user);
    let folded = stage_evidence::fold_into_records(stages, &bounds);
    tracing::info!(
        bounded = bounds.len(),
        folded,
        "sync: inferred clears folded into stage records"
    );
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
    fn cow_level_first_opens_ride_on_the_stage_record() {
        let raw = serde_json::json!({ "user": { "dungeon": { "cowLevel": {
            "spst_08-01": { "id": "spst_08-01", "type": "STAGE", "val": [true], "fts": 1_708_906_788, "rts": 1_708_907_776 },
            "spst_16-01": { "id": "spst_16-01", "type": "STAGE", "val": [], "fts": -1, "rts": -1 },
            "spst_99-01": { "id": "spst_99-01", "type": "STAGE", "val": [true], "fts": 5, "rts": 6 }
        } } } });
        let mut stages = serde_json::json!({
            "spst_08-01": {"stageId": "spst_08-01", "state": 3, "startTimes": 0, "completeTimes": 0},
            "spst_16-01": {"stageId": "spst_16-01", "state": 3, "startTimes": 0, "completeTimes": 0},
            "main_00-01": {"stageId": "main_00-01", "state": 3, "startTimes": 2, "completeTimes": 1}
        });
        merge_cow_level(&mut stages, &raw);
        assert_eq!(stages["spst_08-01"]["cowFirstTs"], 1_708_906_788);
        // Never opened: no annotation. Unknown to the stages map: nothing is invented.
        assert!(stages["spst_16-01"].get("cowFirstTs").is_none());
        assert!(stages.get("spst_99-01").is_none());
        assert!(stages["main_00-01"].get("cowFirstTs").is_none());
        // A payload without the block leaves the map untouched.
        let before = stages.clone();
        merge_cow_level(&mut stages, &serde_json::json!({ "user": {} }));
        assert_eq!(stages, before);
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

    /// The `consumable` block as the game sends it: per item, one instance per
    /// grant with `ts` its expiry (-1 = never) and `count`. Unexpired instances
    /// sum per item, expired ones are dropped, an item with nothing left is
    /// omitted, and the `status` counters become their item ids. Shape read
    /// from a real syncData on 2026-09-20 (95 consumable entries, including
    /// `count: 0` instances and empty objects).
    #[test]
    fn extract_items_merges_inventory_consumables_and_expedited_plans() {
        let far_future = chrono::Utc::now().timestamp() + 86_400;
        let inventory = serde_json::json!({ "30011": 12, "4006": 3 });
        let consumable = serde_json::json!({
            "voucher_elite_II_4": {
                "1": { "ts": -1, "count": 2 },
                "2": { "ts": far_future, "count": 1 },
                "3": { "ts": 1, "count": 7 }
            },
            "voucher_skin": { "9": { "ts": 1, "count": 1 } }
        });
        let items = extract_items(
            &inventory.as_object().cloned(),
            &consumable.as_object().cloned(),
            &[
                (EXPEDITED_PLAN_ITEM, Some(912)),
                (UNIVERSAL_CERTIFICATE_ITEM, Some(513)),
            ],
        );
        let got: std::collections::BTreeMap<String, i64> = items
            .as_array()
            .unwrap()
            .iter()
            .map(|i| {
                (
                    i["item_id"].as_str().unwrap().to_owned(),
                    i["quantity"].as_i64().unwrap(),
                )
            })
            .collect();
        assert_eq!(got["30011"], 12);
        assert_eq!(got["4006"], 3);
        assert_eq!(
            got["voucher_elite_II_4"], 3,
            "two live instances, one expired"
        );
        assert!(
            !got.contains_key("voucher_skin"),
            "only an expired instance"
        );
        assert_eq!(got["7002"], 912, "expedited plans come from status");
        assert_eq!(
            got["classic_normal_ticket"], 513,
            "so do universal certificates"
        );
        assert_eq!(got.len(), 5);
    }

    #[test]
    fn extract_items_without_consumables_or_plans_is_the_inventory() {
        let inventory = serde_json::json!({ "30011": 12 });
        let items = extract_items(&inventory.as_object().cloned(), &None, &[]);
        assert_eq!(items.as_array().unwrap().len(), 1);
        let items = extract_items(&None, &None, &[(EXPEDITED_PLAN_ITEM, Some(0))]);
        assert_eq!(
            items.as_array().unwrap().len(),
            0,
            "a zero counter is not a row"
        );
    }
}
