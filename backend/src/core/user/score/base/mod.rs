//! Base (RIIC) efficiency scoring module
//!
//! Calculates a numerical score for a user's base efficiency based on:
//! - Trading post efficiency and order capacity
//! - Factory production speed and capacity
//! - Power plant electricity output
//! - Dormitory comfort levels
//! - Control center global buffs
//! - Reception room (level-based)
//! - Office (level-based)
//!
//! The scoring focuses on operational efficiency (buff multipliers, speed bonuses)
//! rather than just building levels.

pub mod calculate;
pub mod types;

// Re-export main types and functions for convenience
pub use calculate::calculate_base_score;
pub use types::{
    BaseBreakdown, BaseScore, ControlCenterDetails, ControlCenterScore, DormitoryDetails,
    DormitoryScore, FactoryDetails, FactoryScore, OfficeDetails, OfficeScore, PowerPlantDetails,
    PowerPlantScore, ReceptionRoomDetails, ReceptionRoomScore, TradingPostDetails,
    TradingPostScore,
};
