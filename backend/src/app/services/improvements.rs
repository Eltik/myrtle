use std::collections::{HashMap, HashSet};

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::roster::is_medal_earned;
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::campaign::RotationStatus;
use crate::core::gamedata::types::medal::{MedalData, MedalDefinition, Obtainability};
use crate::core::gamedata::types::operator::{
    Operator, OperatorModule, OperatorProfession, OperatorRarity,
};
use crate::core::gamedata::types::stage_universe::EventEntry;
use crate::core::grade::base::assignment::{
    compute_current_assignment, compute_optimal_assignment, compute_sustained_assignment,
    morale_recovery, sustained_efficiency_of,
};
use crate::core::grade::base::buff_registry::{
    build_name_to_char, build_registry, faction_tags_of,
};
use crate::core::grade::base::types::{
    BaseAssignment, OperatorBaseProfile, RoomAssignment, RotationAssignment, UserBuilding,
};
use crate::core::grade::grade_operators::{
    UpgradeDelta, operator_upgrade_deltas, rarity_to_weight, total_roster_weight,
};
use crate::core::grade::stages::SYNC_GRACE_SECONDS;
use crate::database::models::roster::RosterEntry;
use crate::database::queries::{
    building as building_queries, medals as medal_queries, roguelike as roguelike_queries,
    roster as roster_queries, sandbox as sandbox_queries, stages as stage_queries, users,
};

#[derive(Debug, Clone, Serialize)]
pub struct ImprovementsResponse {
    pub uid: String,
    pub stages: StageImprovements,
    pub roguelike: Vec<RoguelikeThemeImprovement>,
    pub sandbox: SandboxImprovements,
    pub medals: MedalImprovements,
    pub operators: OperatorImprovements,
    pub base: BaseImprovements,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageImprovements {
    pub permanent: StagePoolImprovements,
    pub event: StagePoolImprovements,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StagePoolImprovements {
    pub total: usize,
    pub cleared: usize,
    pub three_starred: usize,
    /// Stages in the user's server-scoped universe that they have not cleared
    /// (state < 2). Sorted by weight desc.
    pub missing: Vec<StageGap>,
    /// Stages cleared but not yet 3-starred. Sorted by weight desc.
    pub not_three_starred: Vec<StageGap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageGap {
    pub stage_id: String,
    pub code: String,
    pub name: Option<String>,
    pub zone_id: String,
    pub weight: f64,
    pub state: i16,
    /// Rotation window for rotating Annihilation maps (`camp_r_*`). `None` for
    /// permanent Annihilation and every non-Annihilation stage. Lets the client
    /// separate the currently-playable rotation from maps that have rotated out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<RotationInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RotationInfo {
    /// "active" (playable now), "past" (rotated out), or "future" (not yet open).
    pub status: &'static str,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoguelikeThemeImprovement {
    pub theme_id: String,
    pub theme_name: String,
    pub endings: ProgressPair,
    pub difficulty: RoguelikeDifficulty,
    pub collectibles: RoguelikeCollectibles,
    pub bp: ProgressPair,
    pub challenges: ProgressPair,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProgressPair {
    pub current: usize,
    pub max: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoguelikeDifficulty {
    pub highest_cleared: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoguelikeCollectibles {
    pub relics: ProgressPair,
    pub capsules: ProgressPair,
    pub bands: ProgressPair,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SandboxImprovements {
    pub achievements: ProgressPair,
    pub nodes: ProgressPair,
    pub tech: ProgressPair,
    pub quests: ProgressPair,
}

#[derive(Debug, Clone, Serialize)]
pub struct MedalImprovements {
    /// Permanent medals (no expiry / latest entry is `PERM:-1`) the user
    /// hasn't earned. Sorted by rarity weight desc.
    pub permanent_missing: Vec<MedalGap>,
    /// Event medals still in their reachable window the user hasn't earned.
    /// Sorted by `end_time` asc (most urgent first).
    pub event_in_window_missing: Vec<MedalGap>,
    /// Medals gated on a collab / one-time operator the player can't reliably
    /// obtain. These are excluded from medal scoring; surfaced separately so the
    /// user understands why they're stuck rather than seeing them as earnable.
    pub operator_locked: Vec<MedalGap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MedalGap {
    pub medal_id: String,
    pub name: String,
    pub rarity: String,
    pub get_method: String,
    pub description: String,
    pub is_hidden: bool,
    /// Event end-timestamp in unix seconds, if this is an event medal.
    pub end_time: Option<i64>,
    /// Set when the medal is locked behind an unobtainable operator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_lock: Option<MedalOperatorLock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalOperatorLock {
    pub operator_id: String,
    pub operator_name: String,
    /// Human-readable reason, e.g. "collab" or "event reward".
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorImprovements {
    /// Owned operators that haven't reached their full investment milestones.
    /// Each entry lists what's still upgradeable. Sorted by rarity desc.
    pub below_milestone: Vec<OperatorGap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorGap {
    pub operator_id: String,
    pub name: String,
    pub rarity: i16,
    pub current_elite: i16,
    pub current_level: i16,
    pub current_skill_level: i16,
    /// Highest mastery the user has on any of this operator's skills (-1 if no skills).
    pub max_mastery: i16,
    /// Highest module level the user has on any advanced module (-1 if no module).
    pub max_module_level: i16,
    /// Current trust percent (0-200), resolved from `favor_point` via the
    /// favor table. Surfaced so the client can show "Trust 87 / 200".
    pub current_trust: f64,
    /// True if this operator is currently published as one of the user's
    /// support units. Support ops are held to the favor table's max trust
    /// (typically 200%) - ordinary ops are "complete" at 100%.
    pub is_support: bool,
    /// Short tags for what's still left, e.g. ["E2", "`MAX_LEVEL`", "M3", "MOD3", "TRUST"].
    pub missing: Vec<&'static str>,
    /// Per-tag projected score gain if the user completed that milestone.
    /// One entry per tag in `missing`, in the same order. See `UpgradeDelta`
    /// for the exact fields - surfaces both the operator-local delta and its
    /// contribution to the user's subscore + `total_score`.
    pub deltas: Vec<UpgradeDelta>,
    /// Combined `operator_grade_delta` if the user did every available upgrade
    /// path on this operator. ELITE (promote + max level at new phase) and
    /// `MAX_LEVEL` (max level at current phase) overlap - only the larger of
    /// the two is counted. All other tags (M3, MOD3, SL7, POT6, TRUST) are
    /// independent and added directly.
    pub subscore_potential_gain: f64,
    /// Same combination as `subscore_potential_gain` but in `total_score`
    /// units - overall grade points the user could still pull from this op.
    pub total_potential_gain: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BaseImprovements {
    /// The player's CURRENT base exactly as stationed right now - for comparing
    /// against the optimized assignments.
    pub current: Option<BaseAssignmentDto>,
    /// The player's current base expressed as a rotation (main + sustained value),
    /// so the comparison can line it up against the optimizer's.
    pub current_rotation: Option<RotationDto>,
    /// Peak assignment - the highest-efficiency arrangement of the roster across
    /// the existing rooms. Useful as a "what's possible right now" view.
    pub optimal: Option<BaseAssignmentDto>,
    /// Staggered rotation for sustained 24/7 operation: a main staffing plus a
    /// backup pool swapped in one operator at a time. `sustained_efficiency` is
    /// the 24/7 output (near peak) and is what the base score uses.
    pub rotation: Option<RotationDto>,
    /// User's room layout (counts + levels per room type)
    pub layout: Vec<RoomLayoutEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseAssignmentDto {
    pub rooms: Vec<RoomAssignmentDto>,
    pub total_production_efficiency: f64,
    /// Realized daily output (the gold→trade loop is coupled: LMD = min(gold
    /// made, gold sold) × 500). This is the value the optimizer maximizes - the
    /// per-room efficiency %s are just for display.
    pub yield_lmd_per_day: f64,
    pub yield_exp_per_day: f64,
    /// LMD-equivalent of everything combined (LMD + EXP at 1:1).
    pub yield_total_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationDto {
    /// The main staffing - your best operators, working almost all the time.
    pub main: BaseAssignmentDto,
    /// Per-room rotation plan: who to swap first, when, and the backup.
    pub rooms: Vec<RoomRotationDto>,
    /// The small shared bench that covers every room: because only one operator is
    /// swapped at a time, a versatile filler can back up several rooms at once.
    pub shared_bench: Vec<AssignedOperator>,
    /// The rotation expressed as a few overlapping staffings to cycle through, so the
    /// whole base is never swapped at once. Consecutive sets share all-but-one
    /// operator per room.
    pub sets: Vec<RotationSetDto>,
    /// Sustained 24/7 output - near peak, reduced only by backup-coverage time.
    pub sustained_efficiency: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationSetDto {
    pub rooms: Vec<RotationSetRoomDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationSetRoomDto {
    pub slot_id: String,
    pub room_type: String,
    /// The operators working this room in this set.
    pub working: Vec<AssignedOperator>,
    /// The main resting this set (covered by the backup), if any.
    pub resting: Option<AssignedOperator>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomRotationDto {
    pub slot_id: String,
    pub room_type: String,
    /// Main operators ordered by who needs swapping first (fastest-draining).
    pub members: Vec<RotationMemberDto>,
    /// The backup to rotate in when a main needs rest.
    pub backup: Option<AssignedOperator>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationMemberDto {
    pub operator: AssignedOperator,
    /// Approximate hours this operator works before you rotate it out.
    pub lasts_hours: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomAssignmentDto {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub formula_type: Option<String>,
    /// Order-acquisition SPEED % (the productivity bonus the game shows).
    pub total_efficiency: f64,
    /// Order-VALUE % (LMD per order, e.g. Proviso) - multiplies LMD yield without
    /// inflating the speed %.
    pub order_value: f64,
    /// True when this is a FIXED synergy squad (operators depend on each other and
    /// can't be swapped); false = flexible / interchangeable team.
    pub locked: bool,
    pub operators: Vec<AssignedOperator>,
    /// Per-room natural yield (trading posts show potential LMD if gold-supplied).
    pub yield_lmd_per_day: f64,
    pub yield_gold_per_day: f64,
    pub yield_exp_per_day: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignedOperator {
    pub operator_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomLayoutEntry {
    pub room_type: String,
    pub count: usize,
    pub levels: Vec<i32>,
}

pub async fn get_improvements(
    state: &AppState,
    uid: &str,
) -> Result<ImprovementsResponse, ApiError> {
    let user = users::find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;
    let user_id = user.id;
    let game_data = state.game_data.load();

    let (roster, supports) = tokio::try_join!(
        roster_queries::get_roster(&state.db, user_id),
        roster_queries::get_supports(&state.db, user_id),
    )?;
    let support_ids: HashSet<&str> = supports.iter().map(|s| s.operator_id.as_str()).collect();
    let owned_operators: HashSet<&str> = roster.iter().map(|e| e.operator_id.as_str()).collect();

    let stages = build_stage_improvements(&state.db, user_id, &game_data).await?;
    let roguelike = build_roguelike_improvements(&state.db, user_id, &game_data).await?;
    let sandbox = build_sandbox_improvements(&state.db, user_id, &game_data).await?;
    let medals = build_medal_improvements(&state.db, user_id, &game_data, &owned_operators).await?;
    let operators = build_operator_improvements(&roster, &game_data, &support_ids);
    let base = build_base_improvements(&state.db, user_id, &roster, &game_data).await?;

    Ok(ImprovementsResponse {
        uid: user.uid,
        stages,
        roguelike,
        sandbox,
        medals,
        operators,
        base,
    })
}

async fn build_stage_improvements(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<StageImprovements, ApiError> {
    let (data, known) = tokio::try_join!(
        stage_queries::get_user_stage_clears(pool, user_id),
        stage_queries::get_known_stage_ids_for_server(pool, user_id),
    )?;
    let clears = &data.clears;
    let last_synced_ts = data.last_synced_ts;
    let universe = &game_data.stage_universe;
    let now = chrono::Utc::now().timestamp();

    let rotation_for = |stage_id: &str| -> Option<RotationInfo> {
        let status = game_data.campaign_rotations.status(stage_id, now)?;
        let window = game_data.campaign_rotations.window(stage_id)?;
        Some(RotationInfo {
            status: match status {
                RotationStatus::Active => "active",
                RotationStatus::Past => "past",
                RotationStatus::Future => "future",
            },
            start_ts: window.start_ts,
            end_ts: window.end_ts,
        })
    };

    let event_in_window = |e: &EventEntry| -> bool {
        if !known.contains(&e.stage_id) {
            return false;
        }
        match (e.start_time, last_synced_ts) {
            (Some(start), Some(sync)) => start <= sync + SYNC_GRACE_SECONDS,
            _ => true,
        }
    };

    let mut permanent = StagePoolImprovements::default();
    for entry in &universe.permanent {
        if !known.contains(&entry.stage_id) {
            continue;
        }
        permanent.total += 1;
        let clear = clears.get(&entry.stage_id);
        let state = clear.map_or(0, |c| c.state);
        let stage_meta = game_data.stages.get(&entry.stage_id);
        let gap = StageGap {
            stage_id: entry.stage_id.clone(),
            code: stage_meta.map(|s| s.code.clone()).unwrap_or_default(),
            name: stage_meta.and_then(|s| s.name.clone()),
            zone_id: stage_meta.map(|s| s.zone_id.clone()).unwrap_or_default(),
            weight: entry.weight,
            state,
            rotation: rotation_for(&entry.stage_id),
        };
        match state {
            s if s >= 3 => {
                permanent.cleared += 1;
                permanent.three_starred += 1;
            }
            s if s >= 2 => {
                permanent.cleared += 1;
                permanent.not_three_starred.push(gap);
            }
            _ => {
                permanent.missing.push(gap);
            }
        }
    }
    sort_by_weight_desc(&mut permanent.missing);
    sort_by_weight_desc(&mut permanent.not_three_starred);

    let mut event = StagePoolImprovements::default();
    for entry in &universe.event {
        if !event_in_window(entry) {
            continue;
        }
        event.total += 1;
        let clear = clears.get(&entry.stage_id);
        let state = clear.map_or(0, |c| c.state);
        let stage_meta = game_data.stages.get(&entry.stage_id);
        let gap = StageGap {
            stage_id: entry.stage_id.clone(),
            code: stage_meta.map(|s| s.code.clone()).unwrap_or_default(),
            name: stage_meta.and_then(|s| s.name.clone()),
            zone_id: stage_meta.map(|s| s.zone_id.clone()).unwrap_or_default(),
            weight: entry.weight,
            state,
            rotation: rotation_for(&entry.stage_id),
        };
        match state {
            s if s >= 3 => {
                event.cleared += 1;
                event.three_starred += 1;
            }
            s if s >= 2 => {
                event.cleared += 1;
                event.not_three_starred.push(gap);
            }
            _ => {
                event.missing.push(gap);
            }
        }
    }
    sort_by_weight_desc(&mut event.missing);
    sort_by_weight_desc(&mut event.not_three_starred);

    Ok(StageImprovements { permanent, event })
}

fn sort_by_weight_desc(items: &mut [StageGap]) {
    items.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.code.cmp(&b.code))
    });
}

async fn build_roguelike_improvements(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<Vec<RoguelikeThemeImprovement>, ApiError> {
    let progress_rows = roguelike_queries::get_roguelike_progress(pool, user_id).await?;
    let progress_by_theme: HashMap<String, &serde_json::Value> = progress_rows
        .iter()
        .map(|(theme_id, json)| (theme_id.clone(), json))
        .collect();

    let mut themes: Vec<RoguelikeThemeImprovement> = Vec::new();
    for (theme_id, theme) in &game_data.roguelike.themes {
        let progress = progress_by_theme.get(theme_id).copied();

        let endings_unlocked = progress
            .and_then(|p| p.get("record"))
            .and_then(|r| r.get("endingCnt"))
            .and_then(|ec| ec.as_object())
            .map_or(0, |obj| {
                let mut ids = HashSet::new();
                for mode in obj.values() {
                    if let Some(mode_obj) = mode.as_object() {
                        for ending_id in mode_obj.keys() {
                            ids.insert(ending_id.clone());
                        }
                    }
                }
                ids.len()
            });

        let highest_difficulty = progress
            .and_then(|p| p.get("collect"))
            .and_then(|c| c.get("modeGrade"))
            .and_then(|mg| mg.get("NORMAL"))
            .and_then(|n| n.as_object())
            .map_or(-1, |obj| {
                obj.iter()
                    .filter(|(_, e)| {
                        e.get("state")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0)
                            >= 2
                    })
                    .filter_map(|(grade_str, _)| grade_str.parse::<i32>().ok())
                    .max()
                    .unwrap_or(-1)
            });

        let count_unlocked = |bucket: &str| -> usize {
            progress
                .and_then(|p| p.get("collect"))
                .and_then(|c| c.get(bucket))
                .and_then(|b| b.as_object())
                .map_or(0, |obj| {
                    obj.values()
                        .filter(|v| {
                            v.get("state")
                                .and_then(serde_json::Value::as_i64)
                                .unwrap_or(0)
                                >= 1
                        })
                        .count()
                })
        };

        let relics_unlocked = count_unlocked("relic");
        let capsules_unlocked = count_unlocked("capsule");
        let bands_unlocked = count_unlocked("band");

        let bp_level = progress
            .and_then(|p| p.get("bp"))
            .and_then(|b| b.get("reward"))
            .and_then(|r| r.as_object())
            .map_or(0, serde_json::Map::len);

        let challenges_completed = progress
            .and_then(|p| p.get("challenge"))
            .and_then(|c| c.get("grade"))
            .and_then(|g| g.as_object())
            .map_or(0, serde_json::Map::len);

        themes.push(RoguelikeThemeImprovement {
            theme_id: theme_id.clone(),
            theme_name: theme.theme_name.clone(),
            endings: ProgressPair {
                current: endings_unlocked,
                max: theme.max_endings as usize,
            },
            difficulty: RoguelikeDifficulty {
                highest_cleared: highest_difficulty,
                max: theme.max_difficulty_grade,
            },
            collectibles: RoguelikeCollectibles {
                relics: ProgressPair {
                    current: relics_unlocked.min(theme.max_relics as usize),
                    max: theme.max_relics as usize,
                },
                capsules: ProgressPair {
                    current: capsules_unlocked.min(theme.max_capsules as usize),
                    max: theme.max_capsules as usize,
                },
                bands: ProgressPair {
                    current: bands_unlocked.min(theme.max_bands as usize),
                    max: theme.max_bands as usize,
                },
            },
            bp: ProgressPair {
                current: bp_level,
                max: theme.max_bp_levels as usize,
            },
            challenges: ProgressPair {
                current: challenges_completed,
                max: theme.max_challenges as usize,
            },
        });
    }

    themes.sort_by(|a, b| a.theme_id.cmp(&b.theme_id));
    Ok(themes)
}

async fn build_sandbox_improvements(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<SandboxImprovements, ApiError> {
    let progress = sandbox_queries::get_user_sandbox_progress(pool, user_id).await?;
    let universe = &game_data.sandbox_universe;

    let Some(sandbox) = progress.as_ref().and_then(|p| {
        p.get("template")
            .and_then(|t| t.get("SANDBOX_V2"))
            .and_then(|t| t.get("sandbox_1"))
    }) else {
        return Ok(SandboxImprovements {
            achievements: ProgressPair {
                current: 0,
                max: universe.max_achievements,
            },
            nodes: ProgressPair {
                current: 0,
                max: universe.max_nodes,
            },
            tech: ProgressPair {
                current: 0,
                max: universe.max_tech_nodes,
            },
            quests: ProgressPair {
                current: 0,
                max: universe.max_quests,
            },
        });
    };

    let achievements = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("achievement"))
        .and_then(|a| a.as_array())
        .map_or(0, std::vec::Vec::len);

    let nodes_explored = sandbox
        .get("main")
        .and_then(|m| m.get("map"))
        .and_then(|m| m.get("node"))
        .and_then(|n| n.as_object())
        .map_or(0, |obj| {
            obj.values()
                .filter(|v| {
                    v.get("state")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(0)
                        >= 1
                })
                .count()
        });

    let tech_unlocked = sandbox
        .get("tech")
        .and_then(|t| t.get("unlock"))
        .and_then(|u| u.as_array())
        .map_or(0, std::vec::Vec::len);

    let quests_completed = sandbox
        .get("collect")
        .and_then(|c| c.get("complete"))
        .and_then(|c| c.get("quest"))
        .and_then(|q| q.as_array())
        .map_or(0, std::vec::Vec::len);

    Ok(SandboxImprovements {
        achievements: ProgressPair {
            current: achievements,
            max: universe.max_achievements,
        },
        nodes: ProgressPair {
            current: nodes_explored,
            max: universe.max_nodes,
        },
        tech: ProgressPair {
            current: tech_unlocked,
            max: universe.max_tech_nodes,
        },
        quests: ProgressPair {
            current: quests_completed,
            max: universe.max_quests,
        },
    })
}

async fn build_medal_improvements(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
    owned_operators: &HashSet<&str>,
) -> Result<MedalImprovements, ApiError> {
    let rows = medal_queries::get_user_medals(pool, user_id).await?;
    let earned: HashSet<String> = rows
        .iter()
        .filter(|(_, val, fts, rts)| {
            let v = val.as_ref().unwrap_or(&serde_json::Value::Null);
            is_medal_earned(v, fts.unwrap_or(0), rts.unwrap_or(0))
        })
        .map(|(id, _, _, _)| id.clone())
        .collect();

    let now = chrono::Utc::now().timestamp();

    let mut permanent_missing: Vec<MedalGap> = Vec::new();
    let mut event_in_window_missing: Vec<MedalGap> = Vec::new();
    let mut operator_locked: Vec<MedalGap> = Vec::new();

    for medal in game_data.medals.medals.values() {
        if earned.contains(&medal.medal_id) {
            continue;
        }
        // Collab-gated medal the user CAN'T currently earn (they don't own the
        // operator). It isn't an "improvement opportunity" - surface it in its
        // own list with operator context instead of the permanent/event gaps.
        // If the user *owns* the collab operator, fall through and treat it as a
        // normal achievable gap.
        if let Some(lock) = game_data.medals.operator_lock(&medal.medal_id)
            && !owned_operators.contains(lock.operator_id.as_str())
        {
            let mut gap = medal_gap(medal, &game_data.medals, None);
            gap.operator_lock = Some(MedalOperatorLock {
                operator_id: lock.operator_id.clone(),
                operator_name: lock.operator_name.clone(),
                reason: "collab",
            });
            operator_locked.push(gap);
            continue;
        }
        match game_data.medals.obtainability(&medal.medal_id, now) {
            Obtainability::Permanent => {
                permanent_missing.push(medal_gap(medal, &game_data.medals, None));
            }
            Obtainability::Event { proxy_close_ts } => {
                // Skip events whose window has already closed - the user can't
                // reach them anymore so it isn't an improvement opportunity.
                // (Past-event medals still affect scoring via the event pool
                // with recency decay - see grade_medals.rs - but they don't
                // belong in the user's "missing" lists.)
                if proxy_close_ts > 0 && proxy_close_ts < now {
                    continue;
                }
                event_in_window_missing.push(medal_gap(
                    medal,
                    &game_data.medals,
                    if proxy_close_ts > 0 {
                        Some(proxy_close_ts)
                    } else {
                        None
                    },
                ));
            }
            #[allow(clippy::needless_continue)]
            // Seasonal/event content not yet reachable (e.g. an SSS tower season
            // that hasn't started). Not an improvement opportunity right now, so
            // it's left out of the lists entirely.
            Obtainability::Unobtainable => continue,
        }
    }

    permanent_missing.sort_by(|a, b| {
        rarity_weight(&b.rarity)
            .partial_cmp(&rarity_weight(&a.rarity))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.medal_id.cmp(&b.medal_id))
    });

    event_in_window_missing.sort_by(|a, b| {
        a.end_time
            .unwrap_or(i64::MAX)
            .cmp(&b.end_time.unwrap_or(i64::MAX))
    });

    operator_locked.sort_by(|a, b| {
        rarity_weight(&b.rarity)
            .partial_cmp(&rarity_weight(&a.rarity))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.medal_id.cmp(&b.medal_id))
    });

    Ok(MedalImprovements {
        permanent_missing,
        event_in_window_missing,
        operator_locked,
    })
}

fn medal_gap(medal: &MedalDefinition, medal_data: &MedalData, end_time: Option<i64>) -> MedalGap {
    MedalGap {
        medal_id: medal.medal_id.clone(),
        name: medal.medal_name.clone(),
        rarity: medal.rarity.clone(),
        get_method: medal.get_method.clone(),
        description: medal_data.resolve_description(&medal.medal_id),
        is_hidden: medal.is_hidden,
        end_time,
        operator_lock: None,
    }
}

fn rarity_weight(rarity: &str) -> f64 {
    // TODO: Keep this in sync with `core/grade/grade_medals.rs`.
    match rarity {
        "T1" => 1.0,
        "T1D5" => 2.5,
        "T2" => 4.0,
        "T2D5" => 10.0,
        "T3" => 20.0,
        "T3D5" => 40.0,
        _ => 1.0,
    }
}

/// Trust percent at which an ordinary operator is considered "complete" - must
/// stay in sync with `core::grade::grade_operators::TRUST_MILESTONE_PCT`.
const TRUST_COMPLETE_PCT: f64 = 100.0;

fn build_operator_improvements(
    roster: &[RosterEntry],
    game_data: &GameData,
    support_ids: &HashSet<&str>,
) -> OperatorImprovements {
    // Total weight across all invested operators - used to translate per-op
    // score deltas into a contribution against operator_grade.
    let total_weight = total_roster_weight(roster, game_data);
    let mut below_milestone: Vec<OperatorGap> = Vec::new();
    for entry in roster {
        let Some(static_op) = game_data.operators.get(&entry.operator_id) else {
            continue;
        };
        if matches!(
            static_op.profession,
            OperatorProfession::Token | OperatorProfession::Trap
        ) || static_op.is_not_obtainable
        {
            continue;
        }

        let max_elite = (static_op.phases.len().saturating_sub(1)) as i16;
        let max_level_at_current_elite = static_op
            .phases
            .get(entry.elite as usize)
            .map_or(0, |p| p.max_level as i16);

        let num_skills = static_op.skills.len();
        let can_master = num_skills > 0 && static_op.phases.len() >= 3;
        let advanced_modules = advanced_modules(static_op);

        let masteries = parse_skill_levels(&entry.masteries);
        let modules = parse_module_levels(&entry.modules);

        let max_mastery = masteries.iter().copied().max().unwrap_or(-1);
        let max_module_level = modules
            .iter()
            .filter(|(id, _)| {
                advanced_modules
                    .iter()
                    .any(|m| m.module.uni_equip_id == *id)
            })
            .map(|(_, lvl)| *lvl)
            .max()
            .unwrap_or(-1);

        let current_trust = game_data.favor.trust_pct(entry.favor_point);
        let max_trust = game_data.favor.max_trust_pct();
        let is_support = support_ids.contains(entry.operator_id.as_str());
        // Support units must reach the table max (typically 200); ordinary ops
        // are done at 100. Clamp the target by max_trust so a stripped favor
        // table doesn't produce an unreachable threshold.
        let trust_target = if is_support {
            max_trust
        } else {
            TRUST_COMPLETE_PCT.min(max_trust)
        };

        let mut missing: Vec<&'static str> = Vec::new();
        if entry.elite < max_elite {
            missing.push("ELITE");
        }
        if max_level_at_current_elite > 0 && entry.level < max_level_at_current_elite {
            missing.push("MAX_LEVEL");
        }
        if can_master {
            if max_mastery < 3 {
                missing.push("M3");
            }
        } else if num_skills > 0 && entry.skill_level < 7 {
            missing.push("SL7");
        }
        if !advanced_modules.is_empty() && max_module_level < 3 {
            missing.push("MOD3");
        }
        if rarity_potential_matters(static_op) && entry.potential < 5 {
            missing.push("POT6");
        }
        if trust_target > 0.0 && current_trust < trust_target {
            missing.push("TRUST");
        }

        if missing.is_empty() {
            continue;
        }

        let rarity_weight = rarity_to_weight(&static_op.rarity);
        let deltas = operator_upgrade_deltas(
            entry,
            static_op,
            &game_data.favor,
            is_support,
            &missing,
            rarity_weight,
            total_weight,
        );
        // ELITE simulates "promote + max level at new phase", which also
        // covers the level dimension that MAX_LEVEL targets. When both tags
        // appear, take whichever delta is larger - additivity would double-
        // count the level dimension.
        let (mut subscore_overlap, mut total_overlap) = (0.0_f64, 0.0_f64);
        let (mut subscore_independent, mut total_independent) = (0.0_f64, 0.0_f64);
        for d in &deltas {
            if d.tag == "ELITE" || d.tag == "MAX_LEVEL" {
                subscore_overlap = subscore_overlap.max(d.operator_grade_delta);
                total_overlap = total_overlap.max(d.total_score_delta);
            } else {
                subscore_independent += d.operator_grade_delta;
                total_independent += d.total_score_delta;
            }
        }
        let subscore_potential_gain = subscore_overlap + subscore_independent;
        let total_potential_gain = total_overlap + total_independent;

        below_milestone.push(OperatorGap {
            operator_id: entry.operator_id.clone(),
            name: static_op.name.clone(),
            rarity: static_op.rarity.to_star_int(),
            current_elite: entry.elite,
            current_level: entry.level,
            current_skill_level: entry.skill_level,
            max_mastery,
            max_module_level,
            current_trust,
            is_support,
            missing,
            deltas,
            subscore_potential_gain,
            total_potential_gain,
        });
    }

    // Within a rarity bucket, the highest-potential gains rank first so users
    // see the most worthwhile upgrades at the top. Ties fall back to op id
    // for a stable order.
    below_milestone.sort_by(|a, b| {
        b.rarity
            .cmp(&a.rarity)
            .then_with(|| {
                b.total_potential_gain
                    .partial_cmp(&a.total_potential_gain)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.operator_id.cmp(&b.operator_id))
    });

    OperatorImprovements { below_milestone }
}

fn advanced_modules(op: &Operator) -> Vec<&OperatorModule> {
    use crate::core::gamedata::types::module::ModuleType;
    op.modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect()
}

const fn rarity_potential_matters(op: &Operator) -> bool {
    !op.can_use_general_potential_item || op.is_sp_char
}

fn parse_skill_levels(masteries_json: &serde_json::Value) -> Vec<i16> {
    let Some(arr) = masteries_json.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|m| {
            m.get("mastery")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i16)
        })
        .collect()
}

fn parse_module_levels(modules_json: &serde_json::Value) -> Vec<(String, i16)> {
    let Some(arr) = modules_json.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|m| {
            let id = m.get("id").and_then(|v| v.as_str())?.to_string();
            let level = m.get("level").and_then(serde_json::Value::as_i64)? as i16;
            Some((id, level))
        })
        .collect()
}

// Re-export the rarity star helper from OperatorRarity for symmetry. (We
// intentionally don't depend on `grade_operators.rs` internals - those are
// scoring-specific and would change the response shape if reused.)
#[allow(dead_code)]
const fn rarity_to_star(rarity: &OperatorRarity) -> i16 {
    rarity.to_star_int()
}

async fn build_base_improvements(
    pool: &PgPool,
    user_id: Uuid,
    roster: &[RosterEntry],
    game_data: &GameData,
) -> Result<BaseImprovements, ApiError> {
    let building_json = building_queries::get_building(pool, user_id).await?;
    let Some(building_json) = building_json else {
        return Ok(BaseImprovements::default());
    };

    let user_building = UserBuilding::from_json(&building_json);
    if user_building.is_empty() {
        return Ok(BaseImprovements::default());
    }

    // Roster → base-skill profiles. Drops operators with no entry in
    // building_data.chars (e.g. tokens, drones).
    let profiles: Vec<OperatorBaseProfile> = roster
        .iter()
        .filter_map(|entry| {
            let bc = game_data.building.chars.get(&entry.operator_id)?;
            let faction_tags = game_data
                .operators
                .get(&entry.operator_id)
                .map(faction_tags_of)
                .unwrap_or_default();
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                &game_data.building,
            ))
        })
        .collect();

    let name_to_char = build_name_to_char(&game_data.operators);
    let (registry, morale_drains) = build_registry(&game_data.building.buffs, &name_to_char);

    let current = compute_current_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
        None,
    );
    let optimal = compute_optimal_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );
    let sustained = compute_sustained_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );

    // The player's current base as a rotation main, so the comparison can show
    // their sustained output against the optimizer's.
    let current_sustained = sustained_efficiency_of(
        &current,
        &profiles,
        &morale_drains,
        morale_recovery(&user_building),
    );
    let current_rotation = Some(rotation_to_dto(
        &RotationAssignment {
            main: current.clone(),
            rooms: Vec::new(),
            shared_bench: Vec::new(),
            sets: Vec::new(),
            sustained_efficiency: current_sustained,
        },
        game_data,
    ));

    let current_dto = base_assignment_to_dto(&current, game_data);
    let optimal_dto = base_assignment_to_dto(&optimal, game_data);
    let rotation_dto = rotation_to_dto(&sustained, game_data);
    let layout = build_layout_summary(&user_building);

    Ok(BaseImprovements {
        current: Some(current_dto),
        current_rotation,
        optimal: Some(optimal_dto),
        rotation: Some(rotation_dto),
        layout,
    })
}

fn base_assignment_to_dto(asn: &BaseAssignment, game_data: &GameData) -> BaseAssignmentDto {
    use crate::core::grade::base::yield_model::BaseFlows;

    // Realized output with the gold→trade coupling (LMD = min(made, sold) × 500).
    let mut flows = BaseFlows::default();
    for r in &asn.rooms {
        flows.add_room(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            r.total_efficiency,
            r.order_value,
        );
    }

    BaseAssignmentDto {
        rooms: asn
            .rooms
            .iter()
            .map(|r| room_assignment_to_dto(r, game_data))
            .collect(),
        total_production_efficiency: asn.total_production_efficiency,
        yield_lmd_per_day: flows.realized_lmd(),
        yield_exp_per_day: flows.exp,
        yield_total_value: flows.total_value(),
    }
}

/// Resolve a `char_id` to an `AssignedOperator` (id + display name).
fn assigned_operator(id: &str, game_data: &GameData) -> AssignedOperator {
    AssignedOperator {
        operator_id: id.to_string(),
        name: game_data
            .operators
            .get(id)
            .map_or_else(|| id.to_string(), |o| o.name.clone()),
    }
}

fn rotation_to_dto(asn: &RotationAssignment, game_data: &GameData) -> RotationDto {
    RotationDto {
        main: base_assignment_to_dto(&asn.main, game_data),
        rooms: asn
            .rooms
            .iter()
            .map(|r| RoomRotationDto {
                slot_id: r.slot_id.clone(),
                room_type: r.room_type.clone(),
                members: r
                    .members
                    .iter()
                    .map(|m| RotationMemberDto {
                        operator: assigned_operator(&m.operator, game_data),
                        lasts_hours: m.lasts_hours,
                    })
                    .collect(),
                backup: r
                    .backup
                    .as_deref()
                    .map(|id| assigned_operator(id, game_data)),
            })
            .collect(),
        shared_bench: asn
            .shared_bench
            .iter()
            .map(|id| assigned_operator(id, game_data))
            .collect(),
        sets: asn
            .sets
            .iter()
            .map(|s| RotationSetDto {
                rooms: s
                    .rooms
                    .iter()
                    .map(|r| RotationSetRoomDto {
                        slot_id: r.slot_id.clone(),
                        room_type: r.room_type.clone(),
                        working: r
                            .working
                            .iter()
                            .map(|id| assigned_operator(id, game_data))
                            .collect(),
                        resting: r
                            .resting
                            .as_deref()
                            .map(|id| assigned_operator(id, game_data)),
                    })
                    .collect(),
            })
            .collect(),
        sustained_efficiency: asn.sustained_efficiency,
    }
}

fn room_assignment_to_dto(room: &RoomAssignment, game_data: &GameData) -> RoomAssignmentDto {
    let y = crate::core::grade::base::yield_model::room_yield(
        &room.room_type,
        room.formula_type.as_deref(),
        room.level,
        room.total_efficiency,
        room.order_value,
    );
    RoomAssignmentDto {
        slot_id: room.slot_id.clone(),
        room_type: room.room_type.clone(),
        level: room.level,
        formula_type: room.formula_type.clone(),
        total_efficiency: room.total_efficiency,
        order_value: room.order_value,
        locked: room.locked,
        operators: room
            .operators
            .iter()
            .map(|id| assigned_operator(id, game_data))
            .collect(),
        yield_lmd_per_day: y.lmd_per_day,
        yield_gold_per_day: y.gold_per_day,
        yield_exp_per_day: y.exp_per_day,
    }
}

fn build_layout_summary(building: &UserBuilding) -> Vec<RoomLayoutEntry> {
    use std::collections::BTreeMap;
    let mut by_type: BTreeMap<String, Vec<i32>> = BTreeMap::new();
    for room in &building.rooms {
        by_type
            .entry(room.room_type.clone())
            .or_default()
            .push(room.level);
    }
    by_type
        .into_iter()
        .map(|(room_type, mut levels)| {
            levels.sort_unstable_by(|a, b| b.cmp(a));
            RoomLayoutEntry {
                count: levels.len(),
                room_type,
                levels,
            }
        })
        .collect()
}
