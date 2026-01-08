//! User account scoring system
//!
//! Calculates a numerical score for a user's account based on:
//! - Operator investment (rarity, level, mastery, modules, skins)
//! - Stage completion (mainline, sidestory, events)

pub mod calculate;
pub mod operators;
pub mod stages;
pub mod types;

// Re-export main types and functions for convenience
pub use calculate::calculate_user_score;
pub use operators::{
    CompletionStatus, MasteryDetails, ModuleDetails, OperatorScore, SkinDetails,
    calculate_operator_score,
};
pub use stages::{ZoneScore, calculate_stage_score};
pub use types::{ScoreBreakdown, UserScore};

// TODO: Add scoring for future aspects:
// - Integrated Strategies (Roguelike) progress
// - Contingency Contract scores
// - Medal collection
// - Base/RIIC efficiency
