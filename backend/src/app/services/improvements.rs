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
use crate::core::gamedata::types::operator::OperatorProfession;
use crate::core::gamedata::types::stage_universe::EventEntry;
use crate::core::grade::base::assignment::{
    cc_non_production_effects, compute_current_assignment, compute_optimal_assignment_with_pins,
    compute_sustained_assignment,
};
use crate::core::grade::base::buff_registry::{
    BuffResolutionStrategy, build_name_to_char, targeted_morale_effects,
};
use crate::core::grade::base::context::BaseContext;
use crate::core::grade::base::dorms::morale_manager_pin;
use crate::core::grade::base::pools::{
    candidate_bundles, has_morale_conditional_grant, plan_optimal_economies,
};
use crate::core::grade::base::shift_rotation::ShiftRotation;
use crate::core::grade::base::shift_rotation::recommend_shift_rotation;
use crate::core::grade::base::sustain_sim::{Verdict, simulate_rotation};
use crate::core::grade::base::types::{
    BaseAssignment, OperatorBaseProfile, RoomAssignment, RotationAssignment, UserBuilding,
};
use crate::core::grade::base::yield_model::room_yield;
use crate::core::grade::grade_medals::rarity_weight;
use crate::core::grade::grade_operators::{
    ScoreDimension, TRUST_MILESTONE_PCT, UpgradeDelta, advanced_module_levels, advanced_modules,
    operator_score_breakdown, operator_upgrade_deltas, potential_matters, rarity_to_weight,
    total_roster_weight,
};
use crate::core::grade::sandbox::grade_sandbox_detail;
use crate::core::grade::sandbox::score::{
    ACHIEVEMENT_WEIGHT, BASE_WEIGHT, CONTENT_WEIGHT, EXPLORATION_WEIGHT, QUEST_WEIGHT, TECH_WEIGHT,
};
use crate::core::grade::stages::{StageClear, event_is_gradeable};
use crate::core::hypergryph::constants::Server;
use crate::database::models::roster::RosterEntry;
use crate::database::queries::building::get_building;
use crate::database::queries::medal_ownership::get_medal_ownership;
use crate::database::queries::medals::get_user_medals;
use crate::database::queries::roguelike::get_roguelike_progress;
use crate::database::queries::roster::get_roster;
use crate::database::queries::roster::get_supports;
use crate::database::queries::stages::get_known_stage_ids_for_server;
use crate::database::queries::stages::get_user_stage_clears;
use crate::database::queries::users::find_by_uid;

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
    /// Overall RA score (0..1) - the weighted sum of every category below. This
    /// is the same value the headline percentage is computed from.
    pub total: f64,
    /// One entry per scored category, in grade-weight order. Each carries its
    /// weight, its own completion, and the concrete counts behind it - so the
    /// breakdown fully accounts for the headline percentage.
    pub categories: Vec<SandboxCategory>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SandboxCategory {
    pub key: &'static str,
    pub label: &'static str,
    /// Fraction of the total RA grade this category contributes (0..1).
    pub weight: f64,
    /// This category's own completion (0..1) - what its bar fills to.
    pub score: f64,
    /// The concrete progress counts that make up this category's score.
    pub parts: Vec<SandboxPart>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SandboxPart {
    pub label: &'static str,
    pub current: usize,
    pub max: usize,
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
    /// Medals whose earnable window has passed and won't reopen: closed-window
    /// event medals plus finished one-time modes / retired towers. Not
    /// actionable, but shown so players can tell what's a dead end vs. still
    /// achievable. Closed-window event medals still contribute to the event pool
    /// with recency decay; one-time / retired medals stay excluded from scoring
    /// (see `build_medal_improvements`). Sorted most-recently-closed first.
    pub unobtainable_missing: Vec<MedalGap>,
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
    /// Share (percent, 0-100) of stat-sharing synced players on the user's
    /// server who have earned this medal - the medal's community rarity.
    /// None until the daily ownership aggregate has been computed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_pct: Option<f64>,
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
    /// Where the current Operators subscore comes from: per-dimension weight
    /// share, completion, and contribution. Contributions sum to the subscore.
    pub score_breakdown: Vec<ScoreDimension>,
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
    /// Peak assignment - the highest-efficiency arrangement of the roster across
    /// the existing rooms. Useful as a "what's possible right now" view.
    pub optimal: Option<BaseAssignmentDto>,
    /// Staggered rotation for sustained 24/7 operation: a main staffing plus a
    /// backup pool swapped in one operator at a time. `sustained_efficiency` is
    /// the 24/7 output (near peak) and is what the base score uses.
    pub rotation: Option<RotationDto>,
    /// User's room layout (counts + levels per room type)
    pub layout: Vec<RoomLayoutEntry>,
    /// Recommended 3-shift rotation paired with the player's saved presets, for the
    /// preset-vs-recommended comparison. `None` when the base has no production rooms.
    pub shift_rotation: Option<ShiftRotationDto>,
    /// The base-wide resource economy plan (Rosmontis / Ebenholz / Mr. Nothing "Perception
    /// Information" system): support operators to station outside production, and the
    /// production operators it powers. `None` unless a 243 roster can field the economy.
    pub perception: Option<PerceptionPlanDto>,
}

/// The base-wide resource economy plan: which support operators to station (and where) to
/// feed the shared resource pool, and which production operators the economy boosts.
#[derive(Debug, Clone, Serialize)]
pub struct PerceptionPlanDto {
    /// Support generators to station outside production (e.g. Mulberry in the HR Office).
    pub support: Vec<PerceptionSupportDto>,
    /// Production operators the economy powers, with the productivity bonus they gain.
    pub consumers: Vec<PerceptionConsumerDto>,
    /// The morale-swap operator (Fiammetta) that sustains the Ling/Dusk morale rotation, when
    /// the plan uses morale-conditional operators and the roster has one.
    pub rotation_manager: Option<AssignedOperator>,
    /// True when the plan needs a morale-swap manager (it uses Ling/Dusk) but the roster has
    /// none - a "you also need a Fiammetta-type operator" flag.
    pub needs_rotation_manager: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerceptionSupportDto {
    pub operator: AssignedOperator,
    pub room_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerceptionConsumerDto {
    pub operator: AssignedOperator,
    pub room_type: String,
    /// Peak bonus (fresh-operator snapshot).
    pub bonus_pct: f64,
    /// Sustained 24/7 bonus (peak x the operator's working uptime).
    pub sustained_pct: f64,
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
    /// Non-production effects this crew provides (Control Center only): clue /
    /// training / HR speed in each boosted facility's OWN units - never folded
    /// into the LMD objective. Empty for other rooms.
    pub non_production: Vec<NonProdEffectDto>,
    /// Per-skill contribution breakdown (evaluate path only; empty elsewhere).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ledger: Vec<SkillLineDto>,
    /// Output-buffer size: orders for a trading post (incl. crew capacity
    /// skills), items for a factory. Evaluate path only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i32>,
    /// Hours from an empty buffer to full - how long the room runs unattended
    /// before it stalls. Evaluate path only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NonProdEffectDto {
    /// The boosted facility's room type ("MEETING", "TRAINING", "HIRE").
    pub room_type: String,
    /// Effect % in that facility's own units (clue collection speed,
    /// Specialization training speed, HR contacting speed).
    pub value: f64,
}

/// One line of a room's per-skill breakdown (the deep dive's "how this is
/// calculated"). Values are MARGINALS in this exact crew - what the room's
/// number loses if this one skill is removed - so pair riders, non-stacking
/// rules and faction gates are already folded in.
#[derive(Debug, Clone, Serialize)]
pub struct SkillLineDto {
    pub operator_id: String,
    /// Display name, so Control-Center lines shown on another room need no
    /// roster join client-side.
    pub operator_name: String,
    pub buff_id: String,
    /// The skill's display name from `building_data`.
    pub buff_name: String,
    /// Marginal speed/efficiency %.
    pub speed_pct: f64,
    /// Marginal order-value %.
    #[serde(skip_serializing_if = "is_zero")]
    pub value_pct: f64,
    /// The line's owner sits in the Control Center, not this room.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub from_control_center: bool,
    /// "contributes" | "inactive" (gate unmet here) | "morale" (moves the
    /// sustain sim, not efficiency) | "capacity" | "`non_production`" |
    /// "unmodeled" (the engine deliberately prices it 0).
    pub disposition: &'static str,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &f64) -> bool {
    v.abs() < 1e-9
}

fn skill_line_dto(
    l: &crate::core::grade::base::skill_ledger::LedgerLine,
    game_data: &GameData,
) -> SkillLineDto {
    use crate::core::grade::base::skill_ledger::LineDisposition as D;
    SkillLineDto {
        operator_id: l.operator_id.clone(),
        operator_name: game_data
            .operators
            .get(&l.operator_id)
            .map_or_else(|| l.operator_id.clone(), |o| o.name.clone()),
        buff_id: l.buff_id.clone(),
        buff_name: game_data
            .building
            .buffs
            .get(&l.buff_id)
            .map_or_else(String::new, |b| b.buff_name.clone()),
        speed_pct: l.speed_pct,
        value_pct: l.value_pct,
        from_control_center: l.from_control_center,
        disposition: match l.disposition {
            D::Contributes => "contributes",
            D::Inactive => "inactive",
            D::MoraleOnly => "morale",
            D::CapacityOnly => "capacity",
            D::NonProduction => "non_production",
            D::Unmodeled => "unmodeled",
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignedOperator {
    pub operator_id: String,
    pub name: String,
    /// True for a SPARE-seat pick (Control-Center bench top-up after every
    /// value-seated operator): parked for lowest opportunity cost, not for its
    /// skills. The frontend badges these so a gated skill text on a
    /// benchwarmer doesn't read as the optimizer's reasoning.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub bench: bool,
}

/// An operator leaving a cell for another room in the SAME shift.
#[derive(Debug, Clone, Serialize)]
pub struct MovedOperator {
    pub operator: AssignedOperator,
    /// Destination room type ("TRADING", "MANUFACTURE", ...).
    pub to_room_type: String,
    /// Destination team/squad label ("Team C", "Squad 2"), when known.
    pub to_team_label: Option<String>,
}

/// A recommended 3-shift rotation alongside the player's saved presets, for the
/// preset-vs-recommended comparison.
#[derive(Debug, Clone, Serialize)]
pub struct ShiftRotationDto {
    pub shifts: Vec<ShiftDto>,
    /// Operators the player runs 24/7 with a morale-swap manager (Fiammetta) - kept working every
    /// shift instead of resting the middle one. The frontend badges these as "24/7 - Fiammetta".
    pub sustained: Vec<AssignedOperator>,
    /// A week-long morale simulation of the recommended rhythm: does it hold up?
    pub sustainability: SustainabilityDto,
}

/// The rotation validated by a time-stepped morale simulation (game-true drain
/// and dorm-recovery rates): honest evidence the plan survives its own rhythm,
/// instead of an unchecked recommendation.
#[derive(Debug, Clone, Serialize)]
pub struct SustainabilityDto {
    /// "`holds_up`" - nobody runs dry; "depletes" - someone's morale hits zero mid-shift.
    pub verdict: String,
    pub horizon_hours: f64,
    /// Operators whose morale empties while working, with when and where.
    pub depleted: Vec<DepletedOperatorDto>,
    /// Peak number of resting operators the dorms could not hold at once.
    pub dorm_overflow: usize,
    /// Every simulated operator's morale over the week, sampled at 12h block
    /// boundaries - the "morale over time" chart. Most-at-risk first.
    pub timeline: Vec<MoraleTimelineDto>,
    /// Per-facility simulated totals over the horizon, with lost hours (dark
    /// shifts + post-depletion time). Absent on older payloads.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub facilities: Vec<FacilityOutputDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FacilityOutputDto {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    /// Simulated totals over the horizon, in the room's own resources.
    pub lmd: f64,
    pub gold: f64,
    pub exp: f64,
    /// Hours of lost work: shifts the room rested dark plus the remainder of
    /// blocks after its crew ran dry.
    pub idle_hours: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoraleTimelineDto {
    pub operator: AssignedOperator,
    /// The room they call home in the rotation (most-worked slot; a permanent
    /// dorm resident's dormitory).
    pub room_type: String,
    pub slot_id: String,
    /// Morale at every 12h boundary, `samples[0]` = t=0 = 24.0.
    pub samples: Vec<f64>,
    /// The final sample - the bar they end the week on.
    pub end: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepletedOperatorDto {
    pub operator: AssignedOperator,
    pub at_hours: f64,
    pub room_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShiftDto {
    /// 1-indexed shift number.
    pub index: usize,
    pub rooms: Vec<ShiftRoomDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShiftRoomDto {
    pub slot_id: String,
    pub room_type: String,
    pub formula_type: Option<String>,
    /// False when this room is deliberately unstaffed this shift (CC's off shift).
    pub active: bool,
    pub recommended: Vec<AssignedOperator>,
    /// The player's saved preset for this room and shift (empty if none).
    pub current: Vec<AssignedOperator>,
    /// Recommended operators the player should ADD (in the recommendation, not the
    /// current preset).
    pub swap_in: Vec<AssignedOperator>,
    /// Current preset operators the player should REMOVE (not in the recommendation).
    pub swap_out: Vec<AssignedOperator>,
    /// Current preset operators who aren't removed but RELOCATE - they're recommended
    /// in a different room this same shift (shown as an add there). Surfaced so the
    /// cell's arithmetic balances instead of an operator silently vanishing.
    pub moved_out: Vec<MovedOperator>,
    /// True when the player's preset already matches the recommendation.
    pub matches: bool,
    /// True when the player's CURRENT team isn't the recommended set but produces within the
    /// leniency band of it (see `gap_pct`), so swapping wouldn't meaningfully help - e.g. a Rhine
    /// operator boosted by Dorothy matching Bryophyta's flat bonus. No swap is suggested.
    pub equivalent: bool,
    /// Signed % gap of the player's team vs the recommendation on the room objective
    /// (negative = the player is slightly behind). Present whenever both teams are scoreable.
    pub gap_pct: Option<f64>,
    /// The recommended crew's room efficiency % (speed incl. global bonuses), for
    /// production/power cells - lets players see how output is distributed across teams.
    pub efficiency: Option<f64>,
    /// Per-skill contribution breakdown for THIS shift's crew (production and
    /// Control-Center cells).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ledger: Vec<SkillLineDto>,
    /// Stable identity of the team/squad staffing this cell - the same id spans the two
    /// consecutive shift columns a production team's 24h block covers.
    pub team_id: Option<String>,
    /// Display label: "Team A/B/C" for production blocks, "Squad 1/2" elsewhere.
    pub team_label: Option<String>,
}

/// A player's team within this fraction of the recommendation's output is "≈ yours" -
/// good enough to keep, with the small gap surfaced as a note instead of a swap nag.
const EQUIVALENT_LENIENCY: f64 = 0.05;

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
    let user = find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;
    let user_id = user.id;
    let game_data = state.default_game_data();

    let (roster, supports) = tokio::try_join!(
        get_roster(&state.db, user_id),
        get_supports(&state.db, user_id),
    )?;
    let support_ids: HashSet<&str> = supports.iter().map(|s| s.operator_id.as_str()).collect();
    let owned_operators: HashSet<&str> = roster.iter().map(|e| e.operator_id.as_str()).collect();

    let stages = build_stage_improvements(&state.db, user_id, &game_data).await?;
    let roguelike = build_roguelike_improvements(&state.db, user_id, &game_data).await?;
    let sandbox = build_sandbox_improvements(&state.db, user_id, &game_data).await?;
    let medals = build_medal_improvements(
        &state.db,
        user_id,
        &user.server,
        &game_data,
        &owned_operators,
    )
    .await?;
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
        get_user_stage_clears(pool, user_id),
        get_known_stage_ids_for_server(pool, user_id),
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

    let event_in_window =
        |e: &EventEntry| -> bool { event_is_gradeable(e, now, last_synced_ts, Some(&known)) };

    let permanent = build_stage_pool(
        universe
            .permanent
            .iter()
            .filter(|e| known.contains(&e.stage_id))
            .map(|e| (e.stage_id.as_str(), e.weight)),
        clears,
        game_data,
        rotation_for,
    );
    let event = build_stage_pool(
        universe
            .event
            .iter()
            .filter(|e| event_in_window(e))
            .map(|e| (e.stage_id.as_str(), e.weight)),
        clears,
        game_data,
        rotation_for,
    );

    Ok(StageImprovements { permanent, event })
}

/// Bucket a pool of stages (already filtered to the gradeable set) into
/// cleared / 3-starred / missing, with gap lists sorted by weight desc.
fn build_stage_pool<'a>(
    entries: impl Iterator<Item = (&'a str, f64)>,
    clears: &HashMap<String, StageClear>,
    game_data: &GameData,
    rotation_for: impl Fn(&str) -> Option<RotationInfo>,
) -> StagePoolImprovements {
    let mut pool = StagePoolImprovements::default();
    for (stage_id, weight) in entries {
        pool.total += 1;
        let state = clears.get(stage_id).map_or(0, |c| c.state);
        let stage_meta = game_data.stages.get(stage_id);
        let gap = StageGap {
            stage_id: stage_id.to_string(),
            code: stage_meta.map(|s| s.code.clone()).unwrap_or_default(),
            name: stage_meta.and_then(|s| s.name.clone()),
            zone_id: stage_meta.map(|s| s.zone_id.clone()).unwrap_or_default(),
            weight,
            state,
            rotation: rotation_for(stage_id),
        };
        match state {
            s if s >= 3 => {
                pool.cleared += 1;
                pool.three_starred += 1;
            }
            s if s >= 2 => {
                pool.cleared += 1;
                pool.not_three_starred.push(gap);
            }
            _ => {
                pool.missing.push(gap);
            }
        }
    }
    sort_by_weight_desc(&mut pool.missing);
    sort_by_weight_desc(&mut pool.not_three_starred);
    pool
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
    let progress_rows = get_roguelike_progress(pool, user_id).await?;
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
    let d = grade_sandbox_detail(pool, user_id, game_data).await?;

    let part = |label: &'static str, current: usize, max: usize| SandboxPart {
        label,
        current,
        max,
    };

    let categories = vec![
        SandboxCategory {
            key: "achievements",
            label: "Achievements",
            weight: ACHIEVEMENT_WEIGHT,
            score: d.achievements,
            parts: vec![part(
                "Completed",
                d.achievements_completed,
                d.achievements_total,
            )],
        },
        SandboxCategory {
            key: "exploration",
            label: "Exploration",
            weight: EXPLORATION_WEIGHT,
            score: d.exploration,
            parts: vec![
                part("Map nodes", d.nodes_explored, d.nodes_total),
                part("Zones", d.zones_unlocked, d.zones_total),
            ],
        },
        SandboxCategory {
            key: "tech",
            label: "Tech unlocks",
            weight: TECH_WEIGHT,
            score: d.tech_tree,
            parts: vec![part("Unlocked", d.tech_unlocked, d.tech_total)],
        },
        SandboxCategory {
            key: "quests",
            label: "Story acts",
            weight: QUEST_WEIGHT,
            score: d.quests,
            parts: vec![part("Completed", d.quests_completed, d.quests_total)],
        },
        SandboxCategory {
            key: "base",
            label: "Base building",
            weight: BASE_WEIGHT,
            score: d.base_building,
            parts: vec![
                part("Base level", d.base_level, d.base_level_max),
                part("Blueprints", d.blueprints, d.blueprints_total),
            ],
        },
        SandboxCategory {
            key: "content",
            label: "Content depth",
            weight: CONTENT_WEIGHT,
            score: d.content_depth,
            parts: vec![
                part("Recipes", d.recipes, d.recipes_total),
                part("Music", d.music, d.music_total),
                part("Rifts cleared", d.rift_levels, d.rift_levels_max),
            ],
        },
    ];

    Ok(SandboxImprovements {
        total: d.total,
        categories,
    })
}

async fn build_medal_improvements(
    pool: &PgPool,
    user_id: Uuid,
    server: &str,
    game_data: &GameData,
    owned_operators: &HashSet<&str>,
) -> Result<MedalImprovements, ApiError> {
    let rows = get_user_medals(pool, user_id).await?;
    let earned: HashSet<String> = rows
        .iter()
        .filter(|(_, val, fts, rts)| {
            let v = val.as_ref().unwrap_or(&serde_json::Value::Null);
            is_medal_earned(v, fts.unwrap_or(0), rts.unwrap_or(0))
        })
        .map(|(id, _, _, _)| id.clone())
        .collect();

    // Community rarity: how many stat-sharing players on this server have
    // earned each medal, from the daily precomputed aggregate. A zero
    // population (job never ran, or unknown server code) yields no percentages
    // rather than misleading ones.
    let (population, ownership_rows) = match Server::parse(server) {
        Some(s) => get_medal_ownership(pool, s.index() as i16).await?,
        None => (0, Vec::new()),
    };
    let ownership: HashMap<&str, i64> = ownership_rows
        .iter()
        .map(|r| (r.medal_id.as_str(), r.owners))
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let owned_pct = |medal_id: &str| -> Option<f64> {
        if population <= 0 {
            return None;
        }
        let owners = ownership.get(medal_id).copied().unwrap_or(0);
        Some((owners as f64 / population as f64 * 1000.0).round() / 10.0)
    };
    let make_gap = |medal: &MedalDefinition, end_time: Option<i64>| {
        medal_gap(
            medal,
            &game_data.medals,
            end_time,
            owned_pct(&medal.medal_id),
        )
    };

    let now = chrono::Utc::now().timestamp();

    let mut permanent_missing: Vec<MedalGap> = Vec::new();
    let mut event_in_window_missing: Vec<MedalGap> = Vec::new();
    let mut operator_locked: Vec<MedalGap> = Vec::new();
    let mut unobtainable_missing: Vec<MedalGap> = Vec::new();

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
            let mut gap = make_gap(medal, None);
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
                permanent_missing.push(make_gap(medal, None));
            }
            Obtainability::Event { proxy_close_ts } => {
                // A closed event window is no longer an improvement opportunity,
                // but it isn't invisible either - route it to the "no longer
                // obtainable" bucket so the user can see it was missed. (Still
                // scored via the event pool with recency decay - see grade_medals.rs.)
                if proxy_close_ts > 0 && proxy_close_ts < now {
                    unobtainable_missing.push(make_gap(medal, Some(proxy_close_ts)));
                    continue;
                }
                event_in_window_missing.push(make_gap(
                    medal,
                    if proxy_close_ts > 0 {
                        Some(proxy_close_ts)
                    } else {
                        None
                    },
                ));
            }
            #[allow(clippy::needless_continue)]
            // Seasonal/event content not reachable *yet* (e.g. an SSS tower season
            // that hasn't started). It will open later, so it isn't an actionable
            // gap - left out of the lists entirely.
            Obtainability::NotYet => continue,
            // Never obtainable again: finished one-time modes / retired towers.
            // Surfaced in the "no longer obtainable" bucket for reference;
            // excluded from scoring entirely (grade_medals.rs).
            Obtainability::Unobtainable => {
                unobtainable_missing.push(make_gap(medal, None));
            }
        }
    }

    permanent_missing.sort_by(cmp_medal_by_rarity_desc);

    event_in_window_missing.sort_by(|a, b| {
        a.end_time
            .unwrap_or(i64::MAX)
            .cmp(&b.end_time.unwrap_or(i64::MAX))
    });

    operator_locked.sort_by(cmp_medal_by_rarity_desc);

    // Most recently closed first; medals with no known close ts (one-time /
    // retired) sort last. Tiebreak by rarity desc then medal id for stability.
    unobtainable_missing.sort_by(|a, b| {
        let a_key = a.end_time.unwrap_or(i64::MIN);
        let b_key = b.end_time.unwrap_or(i64::MIN);
        b_key
            .cmp(&a_key)
            .then_with(|| cmp_medal_by_rarity_desc(a, b))
    });

    Ok(MedalImprovements {
        permanent_missing,
        event_in_window_missing,
        operator_locked,
        unobtainable_missing,
    })
}

fn medal_gap(
    medal: &MedalDefinition,
    medal_data: &MedalData,
    end_time: Option<i64>,
    owned_pct: Option<f64>,
) -> MedalGap {
    MedalGap {
        medal_id: medal.medal_id.clone(),
        name: medal.medal_name.clone(),
        rarity: medal.rarity.clone(),
        get_method: medal.get_method.clone(),
        description: medal_data.resolve_description(&medal.medal_id),
        is_hidden: medal.is_hidden,
        end_time,
        operator_lock: None,
        owned_pct,
    }
}

/// Order medal gaps by rarity weight (desc) so the highest-value gaps rank
/// first, with medal id as a stable tiebreak.
fn cmp_medal_by_rarity_desc(a: &MedalGap, b: &MedalGap) -> std::cmp::Ordering {
    rarity_weight(&b.rarity)
        .partial_cmp(&rarity_weight(&a.rarity))
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| a.medal_id.cmp(&b.medal_id))
}

/// Mastery 3 on the operator's top skill (the "M3" milestone).
const MASTERY_MILESTONE: i16 = 3;
/// Max skill level for operators that can't master (E0/E1 or skill-less-of-mastery ops).
const SKILL_LEVEL_MILESTONE: i16 = 7;
/// Stage 3 on the operator's top advanced module (the "MOD3" milestone).
const MODULE_MILESTONE: i16 = 3;
/// Potential is 0-indexed, so index 5 is pot 6 - full potential (the "POT6" milestone).
const POTENTIAL_MILESTONE_INDEX: i16 = 5;

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
        let max_mastery = masteries.iter().copied().max().unwrap_or(-1);
        let max_module_level = advanced_module_levels(&entry.modules, &advanced_modules)
            .into_iter()
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
            TRUST_MILESTONE_PCT.min(max_trust)
        };

        let mut missing: Vec<&'static str> = Vec::new();
        if entry.elite < max_elite {
            missing.push("ELITE");
        }
        if max_level_at_current_elite > 0 && entry.level < max_level_at_current_elite {
            missing.push("MAX_LEVEL");
        }
        if can_master {
            if max_mastery < MASTERY_MILESTONE {
                missing.push("M3");
            }
        } else if num_skills > 0 && entry.skill_level < SKILL_LEVEL_MILESTONE {
            missing.push("SL7");
        }
        if !advanced_modules.is_empty() && max_module_level < MODULE_MILESTONE {
            missing.push("MOD3");
        }
        if potential_matters(static_op) && entry.potential < POTENTIAL_MILESTONE_INDEX {
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

    OperatorImprovements {
        score_breakdown: operator_score_breakdown(roster, game_data, support_ids),
        below_milestone,
    }
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

async fn build_base_improvements(
    pool: &PgPool,
    user_id: Uuid,
    roster: &[RosterEntry],
    game_data: &GameData,
) -> Result<BaseImprovements, ApiError> {
    let building_json = get_building(pool, user_id).await?;
    let Some(building_json) = building_json else {
        return Ok(BaseImprovements::default());
    };

    let user_building = UserBuilding::from_json(&building_json);
    if user_building.is_empty() {
        return Ok(BaseImprovements::default());
    }

    // Roster → base-skill profiles, buff registry, morale drains. Shared with
    // the interactive planner endpoints so both read a roster the same way.
    let BaseContext {
        profiles,
        mut registry,
        morale_drains,
    } = BaseContext::build(roster, game_data, false);

    // The owner's saved account facts (recruit slots etc.) re-price the same
    // skills here as in the interactive planner - the two surfaces must never
    // disagree on a number.
    if let Ok(Some(value)) = crate::database::queries::users::get_base_facts(pool, user_id).await
        && let Some(slots) = value
            .get("open_recruit_slots")
            .and_then(serde_json::Value::as_u64)
            .filter(|&n| n > 0)
    {
        #[allow(clippy::cast_possible_truncation)]
        let slots = (slots.min(3)) as u32;
        registry = crate::core::grade::base::buff_registry::resolve_account_facts(
            &registry,
            &game_data.building.buffs,
            slots,
        );
    }

    // Evaluate the base-wide resource economies (Rosmontis / Ebenholz / Mr.
    // Nothing "Perception Information" and anything shaped like it) ONCE: each
    // consumer's pool bonus becomes a direct productivity buff in the OPTIMAL
    // registry, so the optimizer values and places them. It's a peak/snapshot
    // strategy (it needs operators resting to feed the pool), so the overrides
    // apply ONLY to `optimal` - not `current`, `sustained`, or the rotation.
    // The machinery reads the actual rooms, so every layout (243, 252, 2/5/2)
    // gets the treatment - the old 243-only gate died with perception.rs.
    // NATIVE-FIRST: the pool machinery (plan_optimal_economies + the bundle
    // oracle below) prices every economy from clauses. The Fiammetta-type
    // morale-swap manager is rotation logistics, not an economy: reserve one
    // whenever the roster owns both a morale-conditional generator (Ling) and
    // a manager - the manager sustains the generator's grant, and she carries
    // no production value a reservation could waste.
    let mut optimal_registry = registry.clone();
    let mut optimal_pins: Vec<(String, String)> = Vec::new();
    let has_conditional_generator = profiles
        .iter()
        .any(|op| has_morale_conditional_grant(op, &game_data.building));
    let manager_pin = morale_manager_pin(&profiles, &user_building, &game_data.building);
    let rotation_manager = manager_pin.as_ref().map(|(id, _)| id.clone());
    if let Some(pin) = manager_pin {
        optimal_pins.push(pin);
    }

    // Native pool economies (Senshi's Monster Meals, Mr. Nothing's and
    // Rosmontis' dorm-fed chains): the same override-and-pin pattern, solved
    // from clauses. A consumer is only credited when every generator feeding
    // it is the consumer themself or pinned by the plan - never phantom value
    // from an operator the search might not seat. Perception's richer
    // economics win any overlap.
    let native_economies =
        plan_optimal_economies(&profiles, &user_building, &game_data.building, &registry);
    for (buff_id, pct) in &native_economies.overrides {
        if optimal_registry.get(buff_id).is_none_or(|s| {
            !matches!(
                s,
                BuffResolutionStrategy::PoolPayoff { .. }
                    | BuffResolutionStrategy::GlobalEffect { .. }
            )
        }) {
            optimal_registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolPayoff { pct: *pct },
            );
        }
    }
    optimal_pins.extend(native_economies.pins.iter().cloned());

    let current = compute_current_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
        None,
    );
    let mut optimal = compute_optimal_assignment_with_pins(
        &profiles,
        &user_building,
        &game_data.building,
        &optimal_registry,
        &morale_drains,
        &optimal_pins,
    );
    // Joint-seating bundles (the Sui Control-Center economy): each bundle
    // packages generator pins + solved consumer overrides, and the OPTIMIZER
    // judges the seat economics - run the search with the bundle and keep it
    // only if the realized total yield improves. Displacement costs (globals
    // the pinned CC seats would otherwise carry) show up in the yield, so no
    // hand-modeled tradeoff is needed.
    for bundle in candidate_bundles(&profiles, &user_building, &game_data.building, &registry) {
        let mut trial_registry = optimal_registry.clone();
        for (buff_id, pct) in &bundle.overrides {
            // Never downgrade: a consumer already priced higher by another
            // plan (native economies, perception) keeps its better value.
            let existing = match trial_registry.get(buff_id) {
                Some(BuffResolutionStrategy::PoolPayoff { pct: p }) => *p,
                _ => f64::NEG_INFINITY,
            };
            if *pct > existing {
                trial_registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPayoff { pct: *pct },
                );
            }
        }
        // Pool-scaled Control-Center globals ride the same never-downgrade
        // rule against whatever global value another plan already folded.
        for (buff_id, target_room, pct) in &bundle.globals {
            let existing = match trial_registry.get(buff_id) {
                Some(BuffResolutionStrategy::GlobalEffect { bonus_pct, .. }) => *bonus_pct,
                _ => f64::NEG_INFINITY,
            };
            if *pct > existing {
                trial_registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::GlobalEffect {
                        target_room: target_room.clone(),
                        bonus_pct: *pct,
                    },
                );
            }
        }
        let mut trial_pins = optimal_pins.clone();
        trial_pins.extend(bundle.pins.iter().cloned());
        let trial = compute_optimal_assignment_with_pins(
            &profiles,
            &user_building,
            &game_data.building,
            &trial_registry,
            &morale_drains,
            &trial_pins,
        );
        use crate::core::grade::base::assignment::assignment_value;
        if assignment_value(&trial.rooms) > assignment_value(&optimal.rooms) + 1e-9 {
            optimal = trial;
            optimal_registry = trial_registry;
            optimal_pins = trial_pins;
        }
    }
    let sustained = compute_sustained_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );

    let current_dto = base_assignment_to_dto(&current, game_data, &profiles, &registry);
    let optimal_dto = base_assignment_to_dto(&optimal, game_data, &profiles, &optimal_registry);
    let rotation_dto = rotation_to_dto(&sustained, game_data);
    let layout = build_layout_summary(&user_building);

    // The shift rotation plans over trading/factory/power structures generically
    // (group tiling, gold-split and power squads all derive from the actual
    // rooms), so any base with the full production spread gets one - 243 and
    // 252 are the layouts the tests pin.
    // The rotation plans with the same economy-aware registry and generator
    // pins as the optimal view (PoolPayoff overrides from perception, native
    // pool plans, and accepted bundles): consumers price their solved payoff
    // in team selection, and pinned generators hold their seats every shift.
    let shift_rotation = if has_shift_rotation_layout(&user_building) {
        let rotation = recommend_shift_rotation(
            &profiles,
            &user_building,
            &game_data.building,
            &optimal_registry,
            &morale_drains,
            &optimal_pins,
        );
        Some(shift_rotation_to_dto(
            &rotation,
            game_data,
            &profiles,
            &user_building,
            &optimal_registry,
            &morale_drains,
        ))
    } else {
        None
    };

    let perception = native_economy_dto(
        &registry,
        &optimal_registry,
        &optimal_pins,
        &profiles,
        &user_building,
        &morale_drains,
        rotation_manager.as_deref(),
        has_conditional_generator,
        game_data,
    );

    Ok(BaseImprovements {
        current: Some(current_dto),
        optimal: Some(optimal_dto),
        rotation: Some(rotation_dto),
        layout,
        shift_rotation,
        perception,
    })
}

/// Build the resource-economy plan DTO from the COMMITTED plan itself: every
/// buff the optimal registry re-priced (pool payoffs, pool-scaled globals)
/// plus the generator seats the plan reserved. `None` when the plan committed
/// no economy. Sustained scales the peak by the mean uptime of the plan's
/// OTHER pinned generators - the pool only stays full while they work; the
/// consumer's own co-present share counts in full. A coarser factor than the
/// old per-contribution weighting, from the same uptime inputs.
#[allow(clippy::too_many_arguments)]
fn native_economy_dto(
    base_registry: &HashMap<String, BuffResolutionStrategy>,
    optimal_registry: &HashMap<String, BuffResolutionStrategy>,
    optimal_pins: &[(String, String)],
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    morale_drains: &HashMap<String, f64>,
    rotation_manager: Option<&str>,
    has_conditional_generator: bool,
    game_data: &GameData,
) -> Option<PerceptionPlanDto> {
    use crate::core::grade::base::assignment::{morale_recovery, op_uptime};

    // The plan's generator seats: every reserved non-production pin except the
    // morale-swap manager (surfaced separately).
    let support: Vec<(String, String)> = {
        let mut seen = std::collections::HashSet::new();
        optimal_pins
            .iter()
            .filter(|(id, room)| {
                Some(id.as_str()) != rotation_manager
                    && room != "MANUFACTURE"
                    && room != "TRADING"
                    && seen.insert(id.clone())
            })
            .cloned()
            .collect()
    };

    // Mean uptime of the OTHER pinned generators, per consumer.
    let recovery = morale_recovery(building);
    let sustain_factor = |consumer: &str| -> f64 {
        let ups: Vec<f64> = support
            .iter()
            .filter(|(id, _)| id != consumer)
            .filter_map(|(id, _)| profiles.iter().find(|p| &p.char_id == id))
            .map(|p| op_uptime(p, morale_drains, recovery))
            .collect();
        if ups.is_empty() {
            1.0
        } else {
            ups.iter().sum::<f64>() / ups.len() as f64
        }
    };

    // Consumers = the optimal registry's economy re-pricings.
    let mut by_char: HashMap<String, PerceptionConsumerDto> = HashMap::new();
    for (buff_id, strategy) in optimal_registry {
        let (room_type, pct) = match strategy {
            BuffResolutionStrategy::PoolPayoff { pct } => {
                let Some(buff) = game_data.building.buffs.get(buff_id) else {
                    continue;
                };
                (buff.room_type.clone(), *pct)
            }
            BuffResolutionStrategy::GlobalEffect {
                target_room,
                bonus_pct,
            } if base_registry.get(buff_id) != Some(strategy) => (target_room.clone(), *bonus_pct),
            _ => continue,
        };
        if pct <= 0.0 {
            continue;
        }
        let Some(owner) = profiles
            .iter()
            .find(|p| p.available_buffs.iter().any(|b| b == buff_id))
        else {
            continue;
        };
        let sustained = pct * sustain_factor(&owner.char_id);
        let slot = by_char
            .entry(owner.char_id.clone())
            .or_insert_with(|| PerceptionConsumerDto {
                operator: assigned_operator(&owner.char_id, game_data),
                room_type: room_type.clone(),
                bonus_pct: 0.0,
                sustained_pct: 0.0,
            });
        if pct > slot.bonus_pct {
            slot.room_type = room_type;
            slot.bonus_pct = pct;
            slot.sustained_pct = sustained;
        }
    }
    if by_char.is_empty() {
        return None;
    }
    let mut consumers: Vec<PerceptionConsumerDto> = by_char.into_values().collect();
    // Strongest bonus first - a stable, meaningful order for the UI.
    consumers.sort_by(|a, b| {
        b.bonus_pct
            .partial_cmp(&a.bonus_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Some(PerceptionPlanDto {
        support: support
            .iter()
            .map(|(id, room)| PerceptionSupportDto {
                operator: assigned_operator(id, game_data),
                room_type: room.clone(),
            })
            .collect(),
        consumers,
        rotation_manager: rotation_manager.map(|id| assigned_operator(id, game_data)),
        needs_rotation_manager: has_conditional_generator && rotation_manager.is_none(),
    })
}

/// A base with the full production spread the shift rotation plans over: at
/// least one trading post, a factory pair to split, and a power plant. Covers
/// 243 and 252 (the tested layouts) and degrades gracefully for others.
fn has_shift_rotation_layout(building: &UserBuilding) -> bool {
    let count = |room_type: &str| {
        building
            .rooms
            .iter()
            .filter(|r| r.room_type == room_type)
            .count()
    };
    count("TRADING") >= 1 && count("MANUFACTURE") >= 2 && count("POWER") >= 1
}

/// For each rotation cell, find the player's CURRENT preset team - across every room of the
/// SAME building type, in any slot or shift - that best matches the recommended team, and pair
/// them so the total number of operator swaps is minimised. This makes the comparison
/// order-independent: as long as the player runs the right team combos somewhere in that
/// building type, it counts as a match regardless of which physical post or shift order they
/// sit in (e.g. teams 1/2/3 across both Trading Posts in any arrangement), while a team combo
/// itself still matters (it is compared as a whole set). Returns `(shift_index, slot_id) ->
/// matched current team`; a recommended cell with no preset left to pair against maps to an
/// empty team (a full swap-in).
fn match_current_teams(rotation: &ShiftRotation) -> HashMap<(usize, String), Vec<String>> {
    struct Cell {
        key: (usize, String),
        set: HashSet<String>,
        ops: Vec<String>,
    }
    // Group by (shift, what the room PRODUCES) - shift index + room type + factory formula. Matching
    // stays order-independent across the SLOTS of one shift (teams 1/2/3 in either Trading Post, in
    // any order), but NOT across shifts: the player's shift-1 preset is compared to the shift-1
    // recommendation, never shuffled into another shift. Grouping by formula also stops a Gold
    // factory's recommendation being matched against an EXP preset (and vice versa).
    type Group = (usize, String, Option<String>);
    let mut rec_by_type: HashMap<Group, Vec<Cell>> = HashMap::new();
    let mut cur_by_type: HashMap<Group, Vec<Cell>> = HashMap::new();
    for shift in &rotation.shifts {
        for room in &shift.rooms {
            let key = (shift.index, room.slot_id.clone());
            let group = (
                shift.index,
                room.room_type.clone(),
                room.formula_type.clone(),
            );
            rec_by_type.entry(group.clone()).or_default().push(Cell {
                key: key.clone(),
                set: room.recommended.iter().cloned().collect(),
                ops: room.recommended.clone(),
            });
            if !room.current.is_empty() {
                cur_by_type.entry(group).or_default().push(Cell {
                    key,
                    set: room.current.iter().cloned().collect(),
                    ops: room.current.clone(),
                });
            }
        }
    }

    let mut out: HashMap<(usize, String), Vec<String>> = HashMap::new();
    for (group, rec_cells) in &rec_by_type {
        let empty: Vec<Cell> = Vec::new();
        let cur_cells = cur_by_type.get(group).unwrap_or(&empty);
        // Rank every (recommended, current) pairing by how few swaps it costs (the symmetric
        // difference of the two teams), then greedily lock in the cheapest pairings - each
        // recommended cell and each preset used at most once. Cost-0 (exact) pairings win
        // first, so a player who already runs the right combos matches no matter the order.
        let mut pairs: Vec<(usize, usize, usize)> = Vec::new();
        for (i, r) in rec_cells.iter().enumerate() {
            for (j, c) in cur_cells.iter().enumerate() {
                let inter = r.set.intersection(&c.set).count();
                let cost = r.set.len() + c.set.len() - 2 * inter;
                pairs.push((cost, i, j));
            }
        }
        pairs.sort_by_key(|p| p.0);
        let mut rec_used = vec![false; rec_cells.len()];
        let mut cur_used = vec![false; cur_cells.len()];
        for (_, i, j) in pairs {
            if rec_used[i] || cur_used[j] {
                continue;
            }
            rec_used[i] = true;
            cur_used[j] = true;
            out.insert(rec_cells[i].key.clone(), cur_cells[j].ops.clone());
        }
        for (i, used) in rec_used.iter().enumerate() {
            if !used {
                out.insert(rec_cells[i].key.clone(), Vec::new());
            }
        }
    }
    out
}

/// Convert a recommended rotation into its DTO, computing the per-room diff between
/// the player's saved preset and the recommendation. A production room whose CURRENT team
/// isn't the recommended set but scores at least as high (an exact tie on the room's objective)
/// is flagged `equivalent` and suggests no swap - the player's team is already as good.
#[allow(clippy::too_many_arguments)]
#[doc(hidden)]
/// Simulate a layout's CURRENT crews with no rotation at all - the "if you
/// never swap" picture adachurch calls the single-form sim. Every staffed room
/// works around the clock at the efficiency the evaluate pass scored it;
/// dormitory occupants rest as permanent residents. The timeline is omitted -
/// the verdict, depletion events and idle hours are the point.
#[allow(clippy::too_many_arguments)]
pub fn static_sustainability(
    building: &UserBuilding,
    assignment: &BaseAssignment,
    profiles: &[OperatorBaseProfile],
    game_data: &GameData,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    initial_morale: Option<&HashMap<String, f64>>,
) -> Option<SustainabilityDto> {
    use crate::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    let efficiency_of: HashMap<&str, f64> = assignment
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r.total_efficiency))
        .collect();
    let cells: Vec<ShiftRoom> = building
        .rooms
        .iter()
        .filter(|r| !r.current_operators.is_empty())
        .map(|r| ShiftRoom {
            slot_id: r.slot_id.clone(),
            room_type: r.room_type.clone(),
            formula_type: r.current_formula.clone(),
            recommended: r.current_operators.clone(),
            current: Vec::new(),
            active: true,
            efficiency: efficiency_of.get(r.slot_id.as_str()).copied(),
            team_id: None,
            team_label: None,
        })
        .collect();
    if cells.is_empty() {
        return None;
    }
    let rotation = ShiftRotation {
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: cells.clone(),
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };
    let targeted = targeted_morale_effects(
        &game_data.building.buffs,
        &build_name_to_char(&game_data.operators),
    );
    let sim = crate::core::grade::base::sustain_sim::simulate_rotation_from(
        &rotation,
        profiles,
        building,
        &game_data.building,
        registry,
        morale_drains,
        &targeted,
        initial_morale,
    );
    let room_type_of = |slot_id: &str| -> String {
        building
            .rooms
            .iter()
            .find(|r| r.slot_id == slot_id)
            .map_or_else(|| slot_id.to_string(), |r| r.room_type.clone())
    };
    Some(SustainabilityDto {
        verdict: match sim.verdict {
            Verdict::HoldsUp => "holds_up".to_string(),
            Verdict::Depletes => "depletes".to_string(),
        },
        horizon_hours: sim.horizon_hours,
        depleted: sim
            .depleted
            .iter()
            .map(|d| DepletedOperatorDto {
                operator: assigned_operator(&d.char_id, game_data),
                at_hours: d.at_hours,
                room_type: room_type_of(&d.slot_id),
            })
            .collect(),
        dorm_overflow: sim.dorm_overflow,
        facilities: sim
            .facilities
            .iter()
            .map(|f| FacilityOutputDto {
                slot_id: f.slot_id.clone(),
                room_type: f.room_type.clone(),
                formula_type: f.formula_type.clone(),
                lmd: f.lmd,
                gold: f.gold,
                exp: f.exp,
                idle_hours: f.idle_hours,
            })
            .collect(),
        timeline: Vec::new(),
    })
}

pub fn shift_rotation_to_dto(
    rotation: &ShiftRotation,
    game_data: &GameData,
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    registry: &std::collections::HashMap<String, BuffResolutionStrategy>,
    morale_drains: &std::collections::HashMap<String, f64>,
) -> ShiftRotationDto {
    use crate::core::grade::base::assignment::team_value;
    let ops = |ids: &[String]| -> Vec<AssignedOperator> {
        ids.iter()
            .map(|id| assigned_operator(id, game_data))
            .collect()
    };
    let bench_ids: std::collections::HashSet<&str> =
        rotation.bench.iter().map(String::as_str).collect();
    // Order-independent pairing of each recommended cell to the player's closest current team.
    let matched = match_current_teams(rotation);
    // The shifts each operator is RECOMMENDED to work (their "home" shifts). A main-team operator
    // works shifts 1 & 3 and rests the middle one; the order-independent overlay can match the
    // player's preset (which keeps running them) into that rest shift and mark it "≈ yours", showing
    // them working a shift the rotation deliberately rests them on - the reported "24/7" operator.
    // So a cell may only KEEP a team containing such an operator on a shift that actually recommends
    // them.
    let mut rec_shifts: HashMap<&str, HashSet<usize>> = HashMap::new();
    for shift in &rotation.shifts {
        for room in shift.rooms.iter().filter(|r| r.active) {
            for id in &room.recommended {
                rec_shifts
                    .entry(id.as_str())
                    .or_default()
                    .insert(shift.index);
            }
        }
    }
    // Per-shift skill ledgers: ablation marginals against the crew that
    // actually works each shift, with that shift's own Control-Center grants.
    let op_index_ledger = crate::core::grade::base::assignment::build_op_index(profiles);
    let facility_counts = crate::core::grade::base::assignment::effective_facility_counts(
        building,
        profiles,
        registry,
        &game_data.building,
    );
    let ledger_ctx = crate::core::grade::base::skill_ledger::LedgerCtx {
        op_index: &op_index_ledger,
        registry,
        building_data: &game_data.building,
        facility_counts: &facility_counts,
        total_dorm_levels: building.total_dorm_levels(),
        morale_drains,
    };

    let mut shift_dtos = Vec::with_capacity(rotation.shifts.len());
    for shift in &rotation.shifts {
        let shift_cc: Vec<String> = shift
            .rooms
            .iter()
            .find(|r| r.room_type == "CONTROL" && r.active)
            .map(|r| r.recommended.clone())
            .unwrap_or_default();
        let (shift_globals, shift_conditions) =
            crate::core::grade::base::skill_ledger::grants_of(&ledger_ctx, &shift_cc);
        // An operator can physically be in only ONE room per shift. The recommended teams are
        // already a conflict-free partition, but the order-independent `equivalent` overlay can
        // surface the player's team (matched from a DIFFERENT slot) for one cell while another
        // cell recommends the same operator - so the same face would show twice in one shift.
        // Each operator is authoritative in the cell that RECOMMENDS it; a cell may only KEEP its
        // current team (mark `equivalent`) when none of that team's operators are recommended
        // elsewhere this shift, nor already kept by an earlier cell. Otherwise it falls back to the
        // real recommendation + swap.
        let rec_owner: HashMap<&str, &str> = shift
            .rooms
            .iter()
            .filter(|r| r.active)
            .flat_map(|r| {
                r.recommended
                    .iter()
                    .map(move |id| (id.as_str(), r.slot_id.as_str()))
            })
            .collect();
        let mut kept: HashSet<String> = HashSet::new();

        let mut room_dtos = Vec::with_capacity(shift.rooms.len());
        for room in &shift.rooms {
            let current = matched
                .get(&(shift.index, room.slot_id.clone()))
                .cloned()
                .unwrap_or_default();
            let rec: HashSet<&str> = room.recommended.iter().map(String::as_str).collect();
            let cur: HashSet<&str> = current.iter().map(String::as_str).collect();
            let mut swap_in: Vec<String> = room
                .recommended
                .iter()
                .filter(|id| !cur.contains(id.as_str()))
                .cloned()
                .collect();
            let mut swap_out: Vec<String> = current
                .iter()
                .filter(|id| !rec.contains(id.as_str()))
                // An operator recommended to WORK in another room this shift isn't being removed -
                // they're relocating to their recommended slot (shown as an add there). Excluding
                // them here avoids showing one operator as both working and leaving the same shift.
                .filter(|id| !matches!(rec_owner.get(id.as_str()), Some(owner) if *owner != room.slot_id))
                .cloned()
                .collect();
            // The relocating operators are surfaced separately with their destination, so
            // the cell's arithmetic balances (a 3-member preset never just loses someone).
            let mut moved_out: Vec<MovedOperator> = current
                .iter()
                .filter(|id| !rec.contains(id.as_str()))
                .filter_map(|id| {
                    let owner = rec_owner.get(id.as_str())?;
                    if *owner == room.slot_id {
                        return None;
                    }
                    let dest = shift.rooms.iter().find(|r| &r.slot_id == owner)?;
                    Some(MovedOperator {
                        operator: assigned_operator(id, game_data),
                        to_room_type: dest.room_type.clone(),
                        to_team_label: dest.team_label.clone(),
                    })
                })
                .collect();
            let mut matches = !current.is_empty()
                && swap_in.is_empty()
                && swap_out.is_empty()
                && moved_out.is_empty();

            // A team the player ALREADY runs that comes within the leniency band of the
            // recommendation's output needs no swap - a factory/trading team (Bryophyta vs a
            // Dorothy-boosted Rhine operator), or a Power Plant specialist with the same
            // drone-recovery % (Pudding vs Indigo, both +15%). The signed gap is surfaced so the
            // UI can note a small improvement exists without nagging a swap. Only checked for
            // rooms whose output `team_value` can score (production + power).
            let mut equivalent = false;
            let mut gap_pct: Option<f64> = None;
            if !current.is_empty()
                && matches!(room.room_type.as_str(), "MANUFACTURE" | "TRADING" | "POWER")
            {
                let f = room.formula_type.as_deref();
                let rec_v = team_value(
                    &room.recommended,
                    &room.room_type,
                    f,
                    profiles,
                    building,
                    &game_data.building,
                    registry,
                    morale_drains,
                );
                let cur_v = team_value(
                    &current,
                    &room.room_type,
                    f,
                    profiles,
                    building,
                    &game_data.building,
                    registry,
                    morale_drains,
                );
                if rec_v > 0.0 {
                    gap_pct = Some((cur_v - rec_v) / rec_v * 100.0);
                }
                // Keeping this team is only valid if it doesn't double-book an operator the shift
                // needs in another room (recommended there, or already kept here), and doesn't keep
                // an operator working a shift the rotation rests them on (recommended on another
                // shift but not this one) - otherwise a main-team operator would appear to work 24/7.
                let conflicts = current.iter().any(|id| {
                    matches!(rec_owner.get(id.as_str()), Some(owner) if *owner != room.slot_id)
                        || kept.contains(id)
                        || rec_shifts
                            .get(id.as_str())
                            .is_some_and(|shifts| !shifts.contains(&shift.index))
                });
                if !matches && cur_v >= rec_v * (1.0 - EQUIVALENT_LENIENCY) && !conflicts {
                    equivalent = true;
                    matches = true;
                    swap_in.clear();
                    swap_out.clear();
                    moved_out.clear();
                }
            }

            // Record the operators this cell tells the player to actually run, so a later cell
            // can't keep a team that reuses one of them.
            if room.active {
                let running = if equivalent || matches {
                    &current
                } else {
                    &room.recommended
                };
                kept.extend(running.iter().cloned());
            }

            let ledger = if !room.active {
                Vec::new()
            } else if room.room_type == "CONTROL" {
                crate::core::grade::base::skill_ledger::control_room_ledger(
                    &ledger_ctx,
                    &room.recommended,
                )
            } else if crate::core::grade::base::util::is_production_room(&room.room_type) {
                crate::core::grade::base::skill_ledger::production_room_ledger(
                    &ledger_ctx,
                    &room.recommended,
                    &room.room_type,
                    room.formula_type.as_deref(),
                    &shift_cc,
                    &shift_globals,
                    &shift_conditions,
                )
            } else {
                Vec::new()
            };
            room_dtos.push(ShiftRoomDto {
                slot_id: room.slot_id.clone(),
                room_type: room.room_type.clone(),
                formula_type: room.formula_type.clone(),
                active: room.active,
                ledger: ledger
                    .iter()
                    .map(|l| skill_line_dto(l, game_data))
                    .collect(),
                recommended: {
                    let mut v = ops(&room.recommended);
                    for o in &mut v {
                        o.bench = bench_ids.contains(o.operator_id.as_str());
                    }
                    v
                },
                current: ops(&current),
                matches,
                equivalent,
                gap_pct,
                efficiency: room.efficiency,
                team_id: room.team_id.clone(),
                team_label: room.team_label.clone(),
                swap_in: ops(&swap_in),
                swap_out: ops(&swap_out),
                moved_out,
            });
        }
        shift_dtos.push(ShiftDto {
            index: shift.index,
            rooms: room_dtos,
        });
    }
    // Validate the recommended rhythm with the game-true morale simulation and
    // ship the verdict alongside the plan.
    let targeted = targeted_morale_effects(
        &game_data.building.buffs,
        &build_name_to_char(&game_data.operators),
    );
    let sim = simulate_rotation(
        rotation,
        profiles,
        building,
        &game_data.building,
        registry,
        morale_drains,
        &targeted,
    );
    let room_type_of = |slot_id: &str| -> String {
        rotation
            .shifts
            .iter()
            .flat_map(|s| s.rooms.iter())
            .find(|r| r.slot_id == slot_id)
            .map_or_else(|| slot_id.to_string(), |r| r.room_type.clone())
    };
    let sustainability = SustainabilityDto {
        verdict: match sim.verdict {
            Verdict::HoldsUp => "holds_up".to_string(),
            Verdict::Depletes => "depletes".to_string(),
        },
        horizon_hours: sim.horizon_hours,
        depleted: sim
            .depleted
            .iter()
            .map(|d| DepletedOperatorDto {
                operator: assigned_operator(&d.char_id, game_data),
                at_hours: d.at_hours,
                room_type: room_type_of(&d.slot_id),
            })
            .collect(),
        dorm_overflow: sim.dorm_overflow,
        facilities: sim
            .facilities
            .iter()
            .map(|f| FacilityOutputDto {
                slot_id: f.slot_id.clone(),
                room_type: f.room_type.clone(),
                formula_type: f.formula_type.clone(),
                lmd: f.lmd,
                gold: f.gold,
                exp: f.exp,
                idle_hours: f.idle_hours,
            })
            .collect(),
        timeline: {
            let mut rows: Vec<MoraleTimelineDto> = sim
                .timeline
                .iter()
                .map(|t| MoraleTimelineDto {
                    operator: assigned_operator(&t.char_id, game_data),
                    room_type: room_type_of(&t.home_slot_id),
                    slot_id: t.home_slot_id.clone(),
                    end: t.samples.last().copied().unwrap_or(0.0),
                    samples: t.samples.clone(),
                })
                .collect();
            // Most-at-risk first: the lowest week-end bar leads the chart.
            rows.sort_by(|a, b| {
                a.end
                    .partial_cmp(&b.end)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.operator.name.cmp(&b.operator.name))
            });
            rows
        },
    };

    ShiftRotationDto {
        shifts: shift_dtos,
        sustained: ops(&rotation.sustained),
        sustainability,
    }
}

pub(crate) fn base_assignment_to_dto(
    asn: &BaseAssignment,
    game_data: &GameData,
    profiles: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> BaseAssignmentDto {
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

    let bench_ids: std::collections::HashSet<&str> = asn.bench.iter().map(String::as_str).collect();
    BaseAssignmentDto {
        rooms: asn
            .rooms
            .iter()
            .map(|r| room_assignment_to_dto(r, game_data, profiles, registry, &bench_ids))
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
        bench: false,
    }
}

fn rotation_to_dto(asn: &RotationAssignment, game_data: &GameData) -> RotationDto {
    RotationDto {
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

fn room_assignment_to_dto(
    room: &RoomAssignment,
    game_data: &GameData,
    profiles: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    bench_ids: &std::collections::HashSet<&str>,
) -> RoomAssignmentDto {
    let y = room_yield(
        &room.room_type,
        room.formula_type.as_deref(),
        room.level,
        room.total_efficiency,
        room.order_value,
    );
    let non_production = if room.room_type == "CONTROL" {
        cc_non_production_effects(&room.operators, profiles, registry)
            .into_iter()
            .map(|(room_type, value)| NonProdEffectDto { room_type, value })
            .collect()
    } else {
        Vec::new()
    };
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
            .map(|id| {
                let mut o = assigned_operator(id, game_data);
                o.bench = bench_ids.contains(id.as_str());
                o
            })
            .collect(),
        yield_lmd_per_day: y.lmd_per_day,
        yield_gold_per_day: y.gold_per_day,
        yield_exp_per_day: y.exp_per_day,
        non_production,
        ledger: room
            .ledger
            .iter()
            .map(|l| skill_line_dto(l, game_data))
            .collect(),
        capacity: room.fill.as_ref().map(|f| f.capacity),
        fill_hours: room.fill.as_ref().map(|f| f.fill_hours),
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

#[cfg(test)]
mod shift_match_tests {
    use super::match_current_teams;
    use crate::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};

    fn room(slot: &str, rec: &[&str], cur: &[&str]) -> ShiftRoom {
        ShiftRoom {
            slot_id: slot.into(),
            room_type: "TRADING".into(),
            formula_type: None,
            recommended: rec.iter().map(|s| (*s).to_string()).collect(),
            current: cur.iter().map(|s| (*s).to_string()).collect(),
            active: true,
            efficiency: None,
            team_id: None,
            team_label: None,
        }
    }
    fn sorted(v: &[String]) -> Vec<String> {
        let mut v = v.to_vec();
        v.sort();
        v
    }

    #[test]
    fn matching_is_order_independent_across_same_type_rooms() {
        // The same two teams, but the player keeps them in the OPPOSITE posts. Order of the
        // physical building must not matter - both posts should read as a perfect match.
        let rotation = ShiftRotation {
            shifts: vec![Shift {
                index: 1,
                rooms: vec![
                    room("a", &["A", "B", "C"], &["D", "E", "F"]),
                    room("b", &["D", "E", "F"], &["A", "B", "C"]),
                ],
            }],
            sustained: vec![],
            bench: vec![],
        };
        let m = match_current_teams(&rotation);
        assert_eq!(
            sorted(&m[&(1, "a".to_string())]),
            vec!["A", "B", "C"],
            "post a pairs with the A/B/C team wherever the player keeps it"
        );
        assert_eq!(
            sorted(&m[&(1, "b".to_string())]),
            vec!["D", "E", "F"],
            "post b pairs with the D/E/F team"
        );
    }

    #[test]
    fn matching_picks_the_closest_team_to_minimise_swaps() {
        // rec [X,Y,Z] should accommodate the player's near-matching [X,Y,Q] (1 swap), not the
        // unrelated [P,Q,R] (3 swaps), even though [P,Q,R] sits in the same physical slot.
        let rotation = ShiftRotation {
            shifts: vec![Shift {
                index: 1,
                rooms: vec![
                    room("a", &["X", "Y", "Z"], &["P", "Q", "R"]),
                    room("b", &["M", "N", "O"], &["X", "Y", "Q"]),
                ],
            }],
            sustained: vec![],
            bench: vec![],
        };
        let m = match_current_teams(&rotation);
        let a = &m[&(1, "a".to_string())];
        assert!(
            a.contains(&"X".to_string()) && a.contains(&"Y".to_string()),
            "post a should pair with the player's near-matching X/Y team, got {a:?}"
        );
    }
}
