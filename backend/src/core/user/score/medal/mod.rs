//! Medal scoring module
//!
//! Calculates scores based on medal completions in Arknights.

pub mod calculate;
pub mod types;

pub use calculate::calculate_medal_score;
pub use types::{MedalBreakdown, MedalCategoryScore, MedalGroupScore, MedalScore};
