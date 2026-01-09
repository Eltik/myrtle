//! Medal scoring types
//!
//! Contains data structures for medal score results and breakdowns.

use serde::{Deserialize, Serialize};

/// Score breakdown for a single medal category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalCategoryScore {
    /// Category key (e.g., "playerMedal", "stageMedal")
    pub category: String,
    /// Display name (e.g., "Records Medal", "Episodes Medal")
    pub category_name: String,
    /// Total score for this category
    pub total_score: f32,
    /// Number of medals earned in this category
    pub medals_earned: i32,
    /// Total medals available in this category
    pub medals_available: i32,
    /// Completion percentage (0-100)
    pub completion_percentage: f32,
    /// T1 (Common) medals earned
    pub t1_earned: i32,
    /// T2 (Uncommon) medals earned
    pub t2_earned: i32,
    /// T3 (Rare) medals earned
    pub t3_earned: i32,
    /// T2D5 (Special) medals earned
    pub t2d5_earned: i32,
}

impl Default for MedalCategoryScore {
    fn default() -> Self {
        Self {
            category: String::new(),
            category_name: String::new(),
            total_score: 0.0,
            medals_earned: 0,
            medals_available: 0,
            completion_percentage: 0.0,
            t1_earned: 0,
            t2_earned: 0,
            t3_earned: 0,
            t2d5_earned: 0,
        }
    }
}

/// Score for a medal group (themed set of medals)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalGroupScore {
    /// Group identifier
    pub group_id: String,
    /// Group display name
    pub group_name: String,
    /// Medals earned in this group
    pub medals_earned: i32,
    /// Total medals in this group
    pub medals_total: i32,
    /// Completion percentage (0-100)
    pub completion_percentage: f32,
    /// Bonus score for completing the entire group
    pub group_bonus: f32,
    /// Whether the group is fully completed
    pub is_complete: bool,
}

/// Overall medal score with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalScore {
    /// Combined total score
    pub total_score: f32,
    /// Score from medal rarity points
    pub rarity_score: f32,
    /// Score from category multipliers
    pub category_bonus_score: f32,
    /// Score from group completion bonuses
    pub group_bonus_score: f32,
    /// Per-category score breakdown
    pub category_scores: Vec<MedalCategoryScore>,
    /// Per-group score breakdown
    pub group_scores: Vec<MedalGroupScore>,
    /// Summary statistics
    pub breakdown: MedalBreakdown,
}

impl Default for MedalScore {
    fn default() -> Self {
        Self {
            total_score: 0.0,
            rarity_score: 0.0,
            category_bonus_score: 0.0,
            group_bonus_score: 0.0,
            category_scores: Vec::new(),
            group_scores: Vec::new(),
            breakdown: MedalBreakdown::default(),
        }
    }
}

/// Summary statistics for medal progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MedalBreakdown {
    // === Total counts ===
    /// Total medals earned
    pub total_medals_earned: i32,
    /// Total medals available in the game
    pub total_medals_available: i32,
    /// Overall completion percentage
    pub total_completion_percentage: f32,

    // === By rarity ===
    /// T1 (Common) medals earned
    pub t1_earned: i32,
    /// T1 (Common) medals available
    pub t1_available: i32,
    /// T2 (Uncommon) medals earned
    pub t2_earned: i32,
    /// T2 (Uncommon) medals available
    pub t2_available: i32,
    /// T3 (Rare) medals earned
    pub t3_earned: i32,
    /// T3 (Rare) medals available
    pub t3_available: i32,
    /// T2D5 (Special) medals earned
    pub t2d5_earned: i32,
    /// T2D5 (Special) medals available
    pub t2d5_available: i32,

    // === By category ===
    /// Records Medal (playerMedal) earned
    pub player_medals: i32,
    /// Episodes Medal (stageMedal) earned
    pub stage_medals: i32,
    /// Annihilation Medal (campMedal) earned
    pub camp_medals: i32,
    /// SSS Medal (towerMedal) earned
    pub tower_medals: i32,
    /// Progress Medal (growthMedal) earned
    pub growth_medals: i32,
    /// Chronicles Medal (storyMedal) earned
    pub story_medals: i32,
    /// Base Medal (buildMedal) earned
    pub build_medals: i32,
    /// Event Medal (activityMedal) earned
    pub activity_medals: i32,
    /// Traveler From Afar Medal (rogueMedal) earned
    pub rogue_medals: i32,
    /// Secret Medal (hiddenMedal) earned
    pub hidden_medals: i32,

    // === Groups ===
    /// Number of medal groups fully completed
    pub groups_complete: i32,
    /// Total number of medal groups
    pub groups_total: i32,
}
