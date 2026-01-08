//! Operator scoring system
//!
//! Calculates a numerical score for individual operators based on investment.
//! Scoring factors include:
//! - Operator rarity (6-star costs significantly more than lower rarities)
//! - Elite promotion and level
//! - Potential rank
//! - Skill masteries (M3 = main milestone)
//! - Module levels (Mod3 = main milestone)
//! - Skin collection (L2D skins worth more, collection completion bonuses)

pub mod calculate;
pub mod helpers;
pub mod types;

// Re-export main function and types for convenience
pub use calculate::calculate_operator_score;
pub use types::{CompletionStatus, MasteryDetails, ModuleDetails, OperatorScore, SkinDetails};
