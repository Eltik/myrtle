//! Sandbox (Reclamation Algorithm) scoring types

use serde::{Deserialize, Serialize};

/// Score breakdown by area
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxAreaScore {
    pub area: i32,
    pub places_score: f32,
    pub nodes_score: f32,
    pub total_score: f32,
    pub places_completed: i32,
    pub places_discovered: i32,
    pub places_total: i32,
    pub nodes_completed: i32,
}

/// Overall sandbox score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxScore {
    pub total_score: f32,
    pub places_score: f32,
    pub nodes_score: f32,
    pub tech_tree_score: f32,
    pub stories_score: f32,
    pub endings_score: f32,
    pub area_scores: Vec<SandboxAreaScore>,
    pub breakdown: SandboxBreakdown,
}

/// Summary breakdown for sandbox progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxBreakdown {
    /// Total places by state
    pub places_completed: i32,
    pub places_discovered: i32,
    pub places_total: i32,
    /// Completion percentages
    pub places_completion_percentage: f32,
    /// Node completion by type
    pub battle_nodes_completed: i32,
    pub choice_nodes_completed: i32,
    pub event_nodes_completed: i32,
    pub ending_nodes_completed: i32,
    pub tech_nodes_completed: i32,
    pub treasure_nodes_completed: i32,
    pub total_nodes_completed: i32,
    /// Tech trees
    pub tech_trees_completed: i32,
    /// Stories
    pub stories_unlocked: i32,
}

impl Default for SandboxScore {
    fn default() -> Self {
        Self {
            total_score: 0.0,
            places_score: 0.0,
            nodes_score: 0.0,
            tech_tree_score: 0.0,
            stories_score: 0.0,
            endings_score: 0.0,
            area_scores: Vec::new(),
            breakdown: SandboxBreakdown::default(),
        }
    }
}
