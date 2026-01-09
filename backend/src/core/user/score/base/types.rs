//! Type definitions for base (RIIC) efficiency scoring
//!
//! Contains types for scoring individual buildings (trading posts, factories,
//! power plants, dormitories, control center, reception room, office) and
//! aggregate base efficiency scores.

use serde::{Deserialize, Serialize};

// ============================================================================
// Trading Post Types
// ============================================================================

/// Score for a single Trading Post
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingPostScore {
    pub slot_id: String,
    pub level: i32,
    pub efficiency_score: f32,
    pub order_score: f32,
    pub strategy_score: f32,
    pub preset_score: f32,
    pub total_score: f32,
    pub details: TradingPostDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingPostDetails {
    /// Speed multiplier from operator skills (1.0 = 100%, 2.0 = 200%)
    pub speed_multiplier: f32,
    /// Current order capacity
    pub stock_limit: i32,
    /// Trading strategy (O_GOLD, O_DIAMOND, etc.)
    pub strategy: String,
    /// Number of operators stationed
    pub operators_stationed: i32,
    /// Number of rotation presets configured
    pub preset_count: i32,
    /// Operators per preset (list of counts)
    pub operators_per_preset: Vec<i32>,
}

// ============================================================================
// Factory Types
// ============================================================================

/// Score for a single Factory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactoryScore {
    pub slot_id: String,
    pub level: i32,
    pub efficiency_score: f32,
    pub capacity_score: f32,
    pub preset_score: f32,
    pub total_score: f32,
    pub details: FactoryDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactoryDetails {
    /// Speed multiplier from operator skills
    pub speed_multiplier: f32,
    /// Output capacity
    pub capacity: i32,
    /// Production type (EXP, gold, originium)
    pub production_type: String,
    /// Number of operators stationed
    pub operators_stationed: i32,
    /// Number of rotation presets configured
    pub preset_count: i32,
    /// Operators per preset (list of counts)
    pub operators_per_preset: Vec<i32>,
}

// ============================================================================
// Power Plant Types
// ============================================================================

/// Score for a single Power Plant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerPlantScore {
    pub slot_id: String,
    pub level: i32,
    pub electricity_score: f32,
    pub drone_score: f32,
    pub total_score: f32,
    pub details: PowerPlantDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerPlantDetails {
    /// Electricity output (60/130/270 based on level)
    pub electricity_output: i32,
    /// Number of operators stationed
    pub operators_stationed: i32,
}

// ============================================================================
// Dormitory Types
// ============================================================================

/// Score for a single Dormitory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DormitoryScore {
    pub slot_id: String,
    pub level: i32,
    pub comfort_score: f32,
    pub morale_recovery_score: f32,
    pub total_score: f32,
    pub details: DormitoryDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DormitoryDetails {
    /// Comfort level (0-5000)
    pub comfort_level: i32,
    /// Comfort as percentage of max (5000)
    pub comfort_percentage: f32,
}

// ============================================================================
// Control Center Types
// ============================================================================

/// Score for the Control Center
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterScore {
    pub slot_id: String,
    pub level: i32,
    pub global_buff_score: f32,
    pub ap_cost_score: f32,
    pub total_score: f32,
    pub details: ControlCenterDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterDetails {
    /// Trading buff percentage from control center
    pub trading_buff: f32,
    /// Manufacture buff percentage from control center
    pub manufacture_buff: f32,
    /// AP cost reduction value
    pub ap_cost_reduction: i32,
    /// Number of operators stationed
    pub operators_stationed: i32,
    /// Number of rotation presets configured
    pub preset_count: i32,
    /// Operators per preset (list of counts)
    pub operators_per_preset: Vec<i32>,
}

// ============================================================================
// Reception Room Types
// ============================================================================

/// Score for the Reception Room (MEETING)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceptionRoomScore {
    pub slot_id: String,
    pub level: i32,
    pub level_score: f32,
    pub operators_score: f32,
    pub total_score: f32,
    pub details: ReceptionRoomDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceptionRoomDetails {
    /// Number of operators stationed
    pub operators_stationed: i32,
}

// ============================================================================
// Office Types
// ============================================================================

/// Score for the Office (HIRE)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeScore {
    pub slot_id: String,
    pub level: i32,
    pub level_score: f32,
    pub operators_score: f32,
    pub total_score: f32,
    pub details: OfficeDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeDetails {
    /// Number of operators stationed
    pub operators_stationed: i32,
}

// ============================================================================
// Aggregate Types
// ============================================================================

/// Overall base efficiency score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseScore {
    pub total_score: f32,

    // Sub-scores by category
    pub trading_score: f32,
    pub factory_score: f32,
    pub power_score: f32,
    pub dormitory_score: f32,
    pub control_center_score: f32,
    pub reception_score: f32,
    pub office_score: f32,
    pub global_bonus_score: f32,

    // Individual room scores
    pub trading_posts: Vec<TradingPostScore>,
    pub factories: Vec<FactoryScore>,
    pub power_plants: Vec<PowerPlantScore>,
    pub dormitories: Vec<DormitoryScore>,
    pub control_center: Option<ControlCenterScore>,
    pub reception_room: Option<ReceptionRoomScore>,
    pub office: Option<OfficeScore>,

    // Summary statistics
    pub breakdown: BaseBreakdown,
}

/// Summary breakdown for base efficiency
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseBreakdown {
    // Building counts
    pub trading_post_count: i32,
    pub factory_count: i32,
    pub power_plant_count: i32,
    pub dormitory_count: i32,

    // Level statistics
    pub max_level_buildings: i32,
    pub total_building_levels: i32,

    // Efficiency metrics
    pub avg_trading_efficiency: f32,
    pub avg_factory_efficiency: f32,
    pub total_electricity_output: i32,
    pub total_electricity_consumption: i32,
    pub electricity_balance: i32,

    // Comfort metrics
    pub total_comfort: i32,
    pub avg_comfort_per_dorm: f32,
    pub max_comfort_dorms: i32,

    // Labor metrics
    pub labor_buff_speed: f32,
    pub labor_max_value: i32,

    // Operator utilization
    pub total_stationed_operators: i32,
    pub operators_in_production: i32,
    pub operators_in_support: i32,
    pub operators_in_rest: i32,
}

impl Default for BaseScore {
    fn default() -> Self {
        Self {
            total_score: 0.0,
            trading_score: 0.0,
            factory_score: 0.0,
            power_score: 0.0,
            dormitory_score: 0.0,
            control_center_score: 0.0,
            reception_score: 0.0,
            office_score: 0.0,
            global_bonus_score: 0.0,
            trading_posts: Vec::new(),
            factories: Vec::new(),
            power_plants: Vec::new(),
            dormitories: Vec::new(),
            control_center: None,
            reception_room: None,
            office: None,
            breakdown: BaseBreakdown::default(),
        }
    }
}
