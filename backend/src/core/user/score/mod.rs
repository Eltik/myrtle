//! User account scoring system
//!
//! Calculates a numerical score for a user's account based on:
//! - Operator investment (rarity, level, mastery, modules, skins)
//! - Stage completion (mainline, sidestory, events)
//! - Integrated Strategies (Roguelike) progress
//! - Reclamation Algorithm (Sandbox) progress
//! - Medal completions
//! - Base (RIIC) efficiency
//! - User grade (S/A/B/C/D/F) based on activity and engagement

pub mod base;
pub mod calculate;
pub mod grade;
pub mod medal;
pub mod operators;
pub mod roguelike;
pub mod sandbox;
pub mod stages;
pub mod types;

// Re-export main types and functions for convenience
pub use base::{
    BaseBreakdown, BaseScore, ControlCenterScore, DormitoryScore, FactoryScore, OfficeScore,
    PowerPlantScore, ReceptionRoomScore, TradingPostScore, calculate_base_score,
};
pub use calculate::calculate_user_score;
pub use grade::{ActivityMetrics, EngagementMetrics, Grade, UserGrade, calculate_user_grade};
pub use medal::{
    MedalBreakdown, MedalCategoryScore, MedalGroupScore, MedalScore, calculate_medal_score,
};
pub use operators::{
    CompletionStatus, MasteryDetails, ModuleDetails, OperatorScore, SkinDetails,
    calculate_operator_score,
};
pub use roguelike::{
    RoguelikeBreakdown, RoguelikeScore, RoguelikeThemeDetails, RoguelikeThemeScore,
    calculate_roguelike_score,
};
pub use sandbox::{SandboxAreaScore, SandboxBreakdown, SandboxScore, calculate_sandbox_score};
pub use stages::{ZoneScore, calculate_stage_score};
pub use types::{ScoreBreakdown, UserScore};
