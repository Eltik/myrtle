//! Roguelike (Integrated Strategies) scoring types

use serde::{Deserialize, Serialize};

/// Score for a single roguelike theme
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeThemeScore {
    pub theme_id: String,
    pub total_score: f32,
    pub endings_score: f32,
    pub bp_score: f32,
    pub buffs_score: f32,
    pub collectibles_score: f32,
    pub challenge_score: f32,
    pub details: RoguelikeThemeDetails,
}

/// Detailed stats for a roguelike theme
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RoguelikeThemeDetails {
    /// Total unique endings unlocked
    pub endings_unlocked: i32,
    /// Total runs by mode
    pub normal_runs: i32,
    pub challenge_runs: i32,
    pub month_team_runs: i32,
    /// Battle pass level achieved (based on rewards claimed)
    pub bp_level: i32,
    /// Accumulated score from buff.score
    pub total_accumulated_score: i64,
    /// Outbuffs unlocked count
    pub buffs_unlocked: i32,
    /// Collectibles
    pub bands_unlocked: i32,
    pub relics_unlocked: i32,
    pub capsules_unlocked: i32,
    /// Challenge mode grades achieved
    pub challenge_grades_achieved: i32,
    pub highest_challenge_grade: i32,
}

/// Overall roguelike score aggregating all themes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeScore {
    pub total_score: f32,
    pub theme_scores: Vec<RoguelikeThemeScore>,
    pub breakdown: RoguelikeBreakdown,
}

/// Summary breakdown for roguelike progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeBreakdown {
    pub themes_played: i32,
    pub total_endings: i32,
    pub total_bp_levels: i32,
    pub total_buffs: i32,
    pub total_collectibles: i32,
    pub total_runs: i32,
}

impl Default for RoguelikeScore {
    fn default() -> Self {
        Self {
            total_score: 0.0,
            theme_scores: Vec::new(),
            breakdown: RoguelikeBreakdown::default(),
        }
    }
}
