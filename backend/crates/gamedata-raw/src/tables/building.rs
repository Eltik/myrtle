use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default};

#[derive(Debug, Default)]
pub struct BuildingTables {
    /// `building_data`
    pub building: BuildingDataFile,
}

/// `building_data.json`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildingDataFile {
    /// All facility buff/skill definitions
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub buffs: HashMap<String, Buff>,

    /// Layout-counted pool resources from term
    #[serde(skip)]
    pub layout_terms: HashMap<String, Vec<String>>,

    /// Operator -> base-skill mappings
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub chars: HashMap<String, BuildingChar>,

    /// Room type definitions
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub rooms: HashMap<String, RoomDef>,

    #[serde(deserialize_with = "deserialize_fb_map_or_default")]
    pub room_unlock_conds: HashMap<String, RoomUnlockCond>,

    pub manufact_data: ManufactData,

    /// Factory recipes (gold, EXP, orundum, chips)
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub manufact_formulas: HashMap<String, ManufactFormula>,

    pub trading_data: TradingData,

    pub control_data: ControlData,

    pub dorm_data: DormData,

    /// Ambience -> recovery conversion:
    /// `comfort / this` = manpower recovered per second
    /// (5000 comfort / 25 = 200 ap/s = +2.0 morale/hr)
    /// 0 disables the bonus
    #[serde(default)]
    pub comfort_manpower_recover_factor: f64,

    pub power_data: PowerData,

    pub hire_data: HireData,

    pub meeting_data: MeetingData,

    pub training_data: TrainingData,

    pub workshop_data: WorkshopData,

    /// Morale cost adjustments by operator count in factories
    #[serde(default)]
    pub manufact_manpower_cost_by_num: Vec<i32>,

    /// Morale cost adjustments by operator count in trading posts
    #[serde(default)]
    pub trading_manpower_cost_by_num: Vec<i32>,

    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub workshop_formulas: HashMap<String, WorkshopFormula>,

    /// Base floorplans, by layout id
    #[serde(deserialize_with = "deserialize_fb_map_or_default", default)]
    pub layouts: HashMap<String, LayoutDef>,

    /// Max Drone capacity before any building built
    #[serde(default)]
    pub init_max_labor: i32,

    /// Furniture catalogue, names and icons for furniture that event shops sell
    #[serde(default)]
    pub custom_data: BuildingCustomData,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildingCustomData {
    #[serde(deserialize_with = "deserialize_fb_map_or_default", default)]
    pub furnitures: HashMap<String, Furniture>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Furniture {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub icon_id: String,
    #[serde(default)]
    pub rarity: i32,
}

/// Base geometry
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LayoutDef {
    pub id: String,
    #[serde(deserialize_with = "deserialize_fb_map_or_default", default)]
    pub slots: HashMap<String, LayoutSlotDef>,
    #[serde(deserialize_with = "deserialize_fb_map_or_default", default)]
    pub storeys: HashMap<String, StoreyDef>,
}

/// Single cell
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LayoutSlotDef {
    pub id: String,
    /// Room class allowed here: `OUTPUT`, `CUSTOM`, `FUNCTION`, `SPECIAL`, or
    /// structure's own `ELEVATOR`/`CORRIDOR`. Matches [`RoomDef::category`]
    pub category: String,
    pub storey_id: String,
    /// Position in half-tiles
    pub offset: RoomSize,
    /// Extent in half-tiles
    pub size: RoomSize,
    /// Max Drone capacity slot adds once built
    #[serde(default)]
    pub provide_labor: i32,
}

/// One floor of base
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StoreyDef {
    pub id: String,
    /// `UPGROUND` for 1F, `DOWNGROUND` for basements
    #[serde(rename = "Type_")]
    pub type_: String,
    /// Control-centre level that unlocks this floor
    pub unlock_control_level: i32,
    /// Rows from the bottom of the base in half-tiles
    /// B4 is 0; 1F is 8
    pub y_offset: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Buff {
    pub buff_id: String,
    pub buff_name: String,
    #[serde(default)]
    pub description: String,
    pub buff_category: String,
    pub room_type: String,
    /// Static efficiency bonus (percentage points). `0` for conditional buffs
    /// whose value depends on context (facility counts, teammates, tags, etc.)
    pub efficiency: i32,
    #[serde(default)]
    pub skill_icon: String,
    #[serde(default)]
    pub sort_id: i32,
    #[serde(default)]
    pub target_group_sort_id: i32,
    /// Types this buff targets (e.g. `["F_GOLD", "F_EXP"]`)
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub buff_color: String,
    #[serde(default)]
    pub buff_icon: String,
    #[serde(default)]
    pub text_color: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildingChar {
    pub char_id: String,
    /// Maximum morale in raw units (divide by 360000 for hours)
    pub max_manpower: i64,
    /// Skill slots that unlock at a different elite/level
    pub buff_char: Vec<BuffCharSlot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuffCharSlot {
    pub buff_data: Vec<BuffDataEntry>,
}

/// A buff granted by an operator at a specific elite/level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuffDataEntry {
    pub buff_id: String,
    pub cond: BuffUnlockCondition,
}

/// The elite + level required to unlock a base skill
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuffUnlockCondition {
    pub phase: String,
    pub level: i32,
}

impl BuffUnlockCondition {
    pub fn elite(&self) -> i32 {
        match self.phase.as_str() {
            "PHASE_0" => 0,
            "PHASE_1" => 1,
            "PHASE_2" => 2,
            _ => 0,
        }
    }
}

/// A facility type definition (e.g. CONTROL, MANUFACTURE, TRADING)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub category: String,
    /// Maximum instances of this room type (-1 = unlimited)
    pub max_count: i32,
    pub can_level_down: bool,
    pub default_prefab_id: String,
    pub size: RoomSize,
    /// Upgrade phases (index = level - 1)
    pub phases: Vec<RoomPhase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomSize {
    pub col: i32,
    pub row: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomPhase {
    pub unlock_cond_id: String,
    pub max_stationed_num: i32,
    /// Power balance. For POWER rooms: generation (60/130/270)
    /// For output/function rooms: consumption (negative)
    /// For CONTROL: 0
    pub electricity: i32,
    /// Base morale cost per cycle
    pub manpower_cost: i32,
    pub build_cost: BuildCost,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildCost {
    pub labor: i32,
    pub time: i32,
    #[serde(default)]
    pub items: Vec<BuildCostItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BuildCostItem {
    pub id: String,
    #[serde(rename = "Type_")]
    pub item_type: String,
    pub count: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomUnlockCond {
    pub id: String,
    /// Requirements keyed by index: facility type + count + level
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub number: HashMap<i32, RoomUnlockReq>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RoomUnlockReq {
    #[serde(rename = "Type_")]
    pub room_type: String,
    pub count: i32,
    pub level: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManufactData {
    /// Base speed buff unit (~0.01 = 1%)
    pub basic_speed_buff: f64,
    pub phases: Vec<ManufactPhase>,
}

/// Factory stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManufactPhase {
    /// Max items in output queue
    pub output_capacity: i32,
    /// Base production speed multiplier
    pub speed: f64,
}

/// A factory recipe definition
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManufactFormula {
    pub formula_id: String,
    /// Product category for buff targeting (e.g. `"F_GOLD"`, `"F_EXP"`)
    pub formula_type: String,
    /// Internal buff type (e.g. `"M_GOLD"`, `"M_EXP"`, `"M_ASC"`)
    pub buff_type: String,
    pub item_id: String,
    /// Production time in seconds
    pub cost_point: i64,
    /// Number of items produced per cycle
    pub count: i32,
    /// Item weight (for capacity calculations)
    pub weight: i32,
    #[serde(default)]
    pub costs: Vec<FormulaCost>,
    #[serde(default)]
    pub require_rooms: Vec<FormulaRoomReq>,
    #[serde(default)]
    pub require_stages: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FormulaCost {
    pub id: String,
    #[serde(rename = "Type_")]
    pub item_type: String,
    pub count: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FormulaRoomReq {
    pub room_id: String,
    pub room_count: i32,
    pub room_level: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradingData {
    /// Base speed buff unit (~0.01 = 1%)
    pub basic_speed_buff: f64,
    pub phases: Vec<TradingPhase>,
}

/// Trading post stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradingPhase {
    /// Max pending orders
    pub order_limit: i32,
    /// Order rarity tier (higher = chance of bigger orders)
    pub order_rarity: i32,
    /// Base order processing speed
    pub order_speed: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ControlData {
    /// Base morale cost reduction for control center operators
    pub basic_cost_buff: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DormData {
    pub phases: Vec<DormPhase>,
}

/// Dormitory stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DormPhase {
    /// Maximum comfort value (furniture limit)
    pub decoration_limit: i32,
    /// Base morale recovery in manpower units per hour
    pub manpower_recover: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerData {
    /// Base drone recovery speed buff unit
    pub basic_speed_buff: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HireData {
    pub basic_speed_buff: f64,
    pub phases: Vec<HirePhase>,
}

/// Office stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HirePhase {
    pub refresh_times: i32,
    pub res_speed: i32,
    pub economize_rate: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingData {
    pub basic_speed_buff: f64,
    pub phases: Vec<MeetingPhase>,
}

/// Reception room stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingPhase {
    pub gathering_speed: i32,
    pub max_visitor_num: i32,
    pub friend_slot_inc: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrainingData {
    pub basic_speed_buff: f64,
    pub phases: Vec<TrainingPhase>,
}

/// Training room stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TrainingPhase {
    /// Max specialization level trainable at this room level
    pub spec_skill_lvl_limit: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkshopData {
    pub phases: Vec<WorkshopPhase>,
}

/// Workshop stats at a given level
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkshopPhase {
    /// Labor efficiency multiplier
    pub manpower_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkshopFormula {
    pub formula_id: String,
    /// Product category (e.g. `"F_EVOLVE"` for material crafting, `"F_ASC"` for
    /// chip conversion, `"F_SKILL"` for skill summaries)
    #[serde(default)]
    pub formula_type: String,
    pub item_id: String,
    pub count: i32,
    pub rarity: i32,
    pub gold_cost: i64,
    pub ap_cost: i64,
    #[serde(default)]
    pub costs: Vec<FormulaCost>,
    #[serde(default)]
    pub require_rooms: Vec<FormulaRoomReq>,
    #[serde(default)]
    pub require_stages: Vec<WorkshopFormulaUnlockStage>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkshopFormulaUnlockStage {
    pub stage_id: String,
    pub rank: i32,
}
