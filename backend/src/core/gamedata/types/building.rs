//! Building (RIIC) game data types.
//!
//! Contains types for parsing `building_data.json` — the static game data that
//! defines base facilities, operator base skills, buff definitions, and
//! production mechanics.

use std::collections::HashMap;

use serde::Deserialize;

use super::serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default};

// ─── Root ────────────────────────────────────────────────────────────────────

/// Root structure for `building_data.json`.
///
/// Only the fields needed for base grading are deserialized; the rest are
/// silently ignored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildingDataFile {
    /// All facility buff/skill definitions (664+).
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub buffs: HashMap<String, Buff>,

    /// Operator -> base-skill mappings (386+).
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub chars: HashMap<String, BuildingChar>,

    /// Room type definitions (12).
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub rooms: HashMap<String, RoomDef>,

    /// Room upgrade prerequisites.
    #[serde(deserialize_with = "deserialize_fb_map_or_default")]
    pub room_unlock_conds: HashMap<String, RoomUnlockCond>,

    /// Factory production mechanics.
    pub manufact_data: ManufactData,

    /// Factory recipes (gold, EXP, orundum, chips).
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub manufact_formulas: HashMap<String, ManufactFormula>,

    /// Trading post mechanics.
    pub trading_data: TradingData,

    /// Control center mechanics.
    pub control_data: ControlData,

    /// Dormitory recovery mechanics.
    pub dorm_data: DormData,

    /// Power plant mechanics.
    pub power_data: PowerData,

    /// Office/recruitment mechanics.
    pub hire_data: HireData,

    /// Reception room mechanics.
    pub meeting_data: MeetingData,

    /// Training room mechanics.
    pub training_data: TrainingData,

    /// Workshop mechanics.
    pub workshop_data: WorkshopData,

    /// Morale cost adjustments by operator count in factories.
    #[serde(default)]
    pub manufact_manpower_cost_by_num: Vec<i32>,

    /// Morale cost adjustments by operator count in trading posts.
    #[serde(default)]
    pub trading_manpower_cost_by_num: Vec<i32>,
}

// ─── Buffs ───────────────────────────────────────────────────────────────────

/// A single base skill/buff definition.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Buff {
    pub buff_id: String,
    pub buff_name: String,
    #[serde(default)]
    pub description: String,
    pub buff_category: String,
    pub room_type: String,
    /// Static efficiency bonus (percentage points). `0` for conditional buffs
    /// whose value depends on context (facility counts, teammates, tags, etc.).
    pub efficiency: i32,
    #[serde(default)]
    pub skill_icon: String,
    #[serde(default)]
    pub sort_id: i32,
    #[serde(default)]
    pub target_group_sort_id: i32,
    /// Product types this buff targets (e.g. `["F_GOLD", "F_EXP"]`).
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub buff_color: String,
    #[serde(default)]
    pub buff_icon: String,
    #[serde(default)]
    pub text_color: String,
}

// ─── Chars (operator base skills) ────────────────────────────────────────────

/// An operator's base skill configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildingChar {
    pub char_id: String,
    /// Maximum morale in raw units (divide by 360000 for hours).
    pub max_manpower: i64,
    /// Skill slots, each unlocking at a different elite/level.
    pub buff_char: Vec<BuffCharSlot>,
}

/// A skill slot containing one or more buff entries.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuffCharSlot {
    pub buff_data: Vec<BuffDataEntry>,
}

/// A buff granted by an operator at a specific promotion/level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuffDataEntry {
    pub buff_id: String,
    pub cond: BuffUnlockCondition,
}

/// The elite phase + level required to unlock a base skill.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuffUnlockCondition {
    pub phase: String,
    pub level: i32,
}

impl BuffUnlockCondition {
    /// Convert the phase string to a numeric elite level.
    /// `"PHASE_0"` -> 0, `"PHASE_1"` -> 1, `"PHASE_2"` -> 2.
    pub fn elite(&self) -> i32 {
        match self.phase.as_str() {
            "PHASE_0" => 0,
            "PHASE_1" => 1,
            "PHASE_2" => 2,
            _ => 0,
        }
    }
}

// ─── Rooms ───────────────────────────────────────────────────────────────────

/// A facility type definition (e.g. CONTROL, MANUFACTURE, TRADING).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub category: String,
    /// Maximum instances of this room type (-1 = unlimited).
    pub max_count: i32,
    pub can_level_down: bool,
    pub default_prefab_id: String,
    pub size: RoomSize,
    /// Upgrade phases (index = level - 1).
    pub phases: Vec<RoomPhase>,
}

/// Grid dimensions for a room.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomSize {
    pub col: i32,
    pub row: i32,
}

/// A single upgrade level for a room.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomPhase {
    pub unlock_cond_id: String,
    /// Max operators that can be stationed.
    pub max_stationed_num: i32,
    /// Power balance. For POWER rooms: generation (60/130/270).
    /// For output/function rooms: consumption (negative).
    /// For CONTROL: 0.
    pub electricity: i32,
    /// Base morale cost per cycle.
    pub manpower_cost: i32,
    pub build_cost: BuildCost,
}

/// Construction cost for upgrading a room.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildCost {
    pub labor: i32,
    pub time: i32,
    #[serde(default)]
    pub items: Vec<BuildCostItem>,
}

/// A material required for construction.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildCostItem {
    pub id: String,
    #[serde(rename = "Type_")]
    pub item_type: String,
    pub count: i32,
}

// ─── Room unlock conditions ──────────────────────────────────────────────────

/// Prerequisites for unlocking a room phase.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomUnlockCond {
    pub id: String,
    /// Requirements keyed by index: facility type + count + level.
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub number: HashMap<i32, RoomUnlockReq>,
}

/// A single prerequisite requirement.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomUnlockReq {
    #[serde(rename = "Type_")]
    pub room_type: String,
    pub count: i32,
    pub level: i32,
}

// ─── Facility mechanics ──────────────────────────────────────────────────────

/// Factory production configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManufactData {
    /// Base speed buff unit (~0.01 = 1%).
    pub basic_speed_buff: f64,
    /// Per-level stats: capacity and base speed.
    pub phases: Vec<ManufactPhase>,
}

/// Factory stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManufactPhase {
    /// Max items in output queue.
    pub output_capacity: i32,
    /// Base production speed multiplier.
    pub speed: f64,
}

/// A factory recipe definition.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManufactFormula {
    pub formula_id: String,
    /// Product category for buff targeting (e.g. `"F_GOLD"`, `"F_EXP"`).
    pub formula_type: String,
    /// Internal buff type (e.g. `"M_GOLD"`, `"M_EXP"`, `"M_ASC"`).
    pub buff_type: String,
    /// Produced item ID.
    pub item_id: String,
    /// Production time in seconds.
    pub cost_point: i64,
    /// Number of items produced per cycle.
    pub count: i32,
    /// Item weight (for capacity calculations).
    pub weight: i32,
    /// Input materials required.
    #[serde(default)]
    pub costs: Vec<FormulaCost>,
    /// Room requirements to unlock this formula.
    #[serde(default)]
    pub require_rooms: Vec<FormulaRoomReq>,
    /// Stage requirements to unlock this formula.
    #[serde(default)]
    pub require_stages: Vec<serde_json::Value>,
}

/// Material cost for a factory formula.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FormulaCost {
    pub id: String,
    #[serde(rename = "Type_")]
    pub item_type: String,
    pub count: i32,
}

/// Room requirement to unlock a factory formula.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FormulaRoomReq {
    pub room_id: String,
    pub room_count: i32,
    pub room_level: i32,
}

/// Trading post configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradingData {
    /// Base speed buff unit (~0.01 = 1%).
    pub basic_speed_buff: f64,
    /// Per-level stats: order limit, rarity, speed.
    pub phases: Vec<TradingPhase>,
}

/// Trading post stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradingPhase {
    /// Max pending orders.
    pub order_limit: i32,
    /// Order rarity tier (higher = chance of bigger orders).
    pub order_rarity: i32,
    /// Base order processing speed.
    pub order_speed: f64,
}

/// Control center configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ControlData {
    /// Base morale cost reduction for control center operators.
    pub basic_cost_buff: i32,
}

/// Dormitory recovery configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DormData {
    /// Per-level stats: comfort limit and morale recovery rate.
    pub phases: Vec<DormPhase>,
}

/// Dormitory stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DormPhase {
    /// Maximum comfort value (furniture limit).
    pub decoration_limit: i32,
    /// Base morale recovery in manpower units per hour.
    pub manpower_recover: i32,
}

/// Power plant configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerData {
    /// Base drone recovery speed buff unit.
    pub basic_speed_buff: f64,
}

/// Office/recruitment configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HireData {
    pub basic_speed_buff: f64,
    pub phases: Vec<HirePhase>,
}

/// Office stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HirePhase {
    pub refresh_times: i32,
    pub res_speed: i32,
    pub economize_rate: f64,
}

/// Reception room configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingData {
    pub basic_speed_buff: f64,
    pub phases: Vec<MeetingPhase>,
}

/// Reception room stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingPhase {
    pub gathering_speed: i32,
    pub max_visitor_num: i32,
    pub friend_slot_inc: i32,
}

/// Training room configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrainingData {
    pub basic_speed_buff: f64,
    pub phases: Vec<TrainingPhase>,
}

/// Training room stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrainingPhase {
    /// Max specialization level trainable at this room level.
    pub spec_skill_lvl_limit: i32,
}

/// Workshop configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkshopData {
    pub phases: Vec<WorkshopPhase>,
}

/// Workshop stats at a given level.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkshopPhase {
    /// Labor efficiency multiplier.
    pub manpower_factor: f64,
}
