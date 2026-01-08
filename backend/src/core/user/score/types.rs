//! Shared types for user account scoring
//!
//! Contains the main UserScore and ScoreBreakdown types that aggregate
//! scores from operators, stages, and other scoring modules.

use serde::{Deserialize, Serialize};

use super::operators::types::OperatorScore;
use super::stages::types::ZoneScore;

/// Total account score with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserScore {
    /// Combined total score (operators + stages + ...)
    pub total_score: f32,
    /// Score from operator investment only
    pub operator_score: f32,
    /// Score from stage completion only
    pub stage_score: f32,
    /// Individual operator scores sorted by total descending
    pub operator_scores: Vec<OperatorScore>,
    /// Zone/chapter completion scores
    pub zone_scores: Vec<ZoneScore>,
    /// Summary statistics
    pub breakdown: ScoreBreakdown,
}

/// Summary statistics for the account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreBreakdown {
    // === Operator Stats ===
    pub total_operators: i32,
    pub six_star_count: i32,
    pub five_star_count: i32,
    pub four_star_count: i32,
    pub three_star_and_below_count: i32,
    pub m9_count: i32,
    pub m3_count: i32,
    pub e2_count: i32,
    pub average_score_per_operator: f32,
    /// Total skins owned across all operators
    pub total_skins_owned: i32,
    /// Operators with full skin collection (100%)
    pub full_skin_collection_count: i32,

    // === Stage Stats ===
    /// Main story completion percentage (0-100)
    pub mainline_completion: f32,
    /// Side story completion percentage (0-100)
    pub sidestory_completion: f32,
    /// Event/activity completion percentage (0-100)
    pub activity_completion: f32,
    /// Total stages completed across all zones
    pub total_stages_completed: i32,
    /// Total stages available in the game
    pub total_stages_available: i32,
    /// Total perfect/3-star clears
    pub total_perfect_clears: i32,
}

impl Default for ScoreBreakdown {
    fn default() -> Self {
        Self {
            // Operator stats
            total_operators: 0,
            six_star_count: 0,
            five_star_count: 0,
            four_star_count: 0,
            three_star_and_below_count: 0,
            m9_count: 0,
            m3_count: 0,
            e2_count: 0,
            average_score_per_operator: 0.0,
            total_skins_owned: 0,
            full_skin_collection_count: 0,
            // Stage stats
            mainline_completion: 0.0,
            sidestory_completion: 0.0,
            activity_completion: 0.0,
            total_stages_completed: 0,
            total_stages_available: 0,
            total_perfect_clears: 0,
        }
    }
}

impl Default for UserScore {
    fn default() -> Self {
        Self {
            total_score: 0.0,
            operator_score: 0.0,
            stage_score: 0.0,
            operator_scores: Vec::new(),
            zone_scores: Vec::new(),
            breakdown: ScoreBreakdown::default(),
        }
    }
}
