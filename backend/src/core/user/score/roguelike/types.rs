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
    pub difficulty_score: f32,
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
    /// Challenges cleared at max difficulty (grade 2)
    pub grade_2_challenges: i32,

    // === Max Values (from game data) ===
    /// Max endings available for this theme
    pub max_endings: i32,
    /// Max relics available for this theme
    pub max_relics: i32,
    /// Max capsules available for this theme
    pub max_capsules: i32,
    /// Max bands available for this theme
    pub max_bands: i32,
    /// Max challenges available for this theme
    pub max_challenges: i32,
    /// Max monthly squads available for this theme
    pub max_monthly_squads: i32,
    /// Highest difficulty grade available for this theme
    pub max_difficulty_grade: i32,

    // === Completion Percentages ===
    /// Endings completion percentage (0-100)
    pub endings_completion_percentage: f32,
    /// Relics completion percentage (0-100)
    pub relics_completion_percentage: f32,
    /// Capsules completion percentage (0-100)
    pub capsules_completion_percentage: f32,
    /// Bands completion percentage (0-100)
    pub bands_completion_percentage: f32,
    /// Total collectibles (relics + capsules + bands) completion percentage (0-100)
    pub collectibles_completion_percentage: f32,
    /// Challenges at max grade (grade 2) completion percentage (0-100)
    pub challenges_at_max_grade_percentage: f32,
    /// Overall theme completion percentage (0-100)
    pub overall_completion_percentage: f32,
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
    /// Total challenges cleared at max difficulty (grade 2) across all themes
    pub total_grade_2_challenges: i32,
    /// Number of themes with at least one grade 2 clear
    pub themes_at_max_difficulty: i32,

    // === Max Totals (from game data) ===
    /// Total themes available in the game
    pub total_themes_available: i32,
    /// Total endings available across all themes
    pub total_max_endings: i32,
    /// Total collectibles (relics + capsules + bands) available across all themes
    pub total_max_collectibles: i32,
    /// Total challenges available across all themes
    pub total_max_challenges: i32,
    /// Total monthly squads available across all themes
    pub total_max_monthly_squads: i32,

    // === Completion Percentages ===
    /// Themes played completion percentage (0-100)
    pub themes_completion_percentage: f32,
    /// Endings completion percentage (0-100)
    pub endings_completion_percentage: f32,
    /// Collectibles (relics + capsules + bands) completion percentage (0-100)
    pub collectibles_completion_percentage: f32,
    /// Challenges at max grade (grade 2) completion percentage (0-100)
    pub challenges_completion_percentage: f32,
    /// Overall roguelike completion percentage (0-100)
    pub overall_completion_percentage: f32,
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
