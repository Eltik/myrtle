//! User account scoring system
//!
//! Calculates a numerical score for a user's account based on:
//! - Operator investment (rarity, level, mastery, modules, skins)
//! - Stage completion (mainline, sidestory, events)
//! - Integrated Strategies (Roguelike) progress
//! - Reclamation Algorithm (Sandbox) progress
//! - Medal completions

pub mod calculate;
pub mod medal;
pub mod operators;
pub mod roguelike;
pub mod sandbox;
pub mod stages;
pub mod types;

// Re-export main types and functions for convenience
pub use calculate::calculate_user_score;
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
