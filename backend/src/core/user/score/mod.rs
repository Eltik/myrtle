//! User account scoring system
//!
//! Calculates a numerical score for a user's account based on operator investment.

pub mod calculate;
pub mod operators;

// Re-export main types and functions for convenience
pub use calculate::calculate_user_score;
pub use operators::{
    CompletionStatus, MasteryDetails, ModuleDetails, OperatorScore, ScoreBreakdown, UserScore,
    calculate_operator_score,
};

// TODO: Add scoring for future aspects:
// - Trust level (favor_point)
// - Skin collection (vanity score)
// - Stage completion
// - Integrated Strategies (Roguelike) progress
// - Contingency Contract scores
// - Event completion
// - Medal collection
// - Base/RIIC efficiency
