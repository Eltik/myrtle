//! Stage scoring types
//!
//! Contains types for tracking stage/zone completion and scoring.

use serde::{Deserialize, Serialize};

use crate::core::local::types::zone::ZoneType;

/// Score result for a single zone/chapter/event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneScore {
    pub zone_id: String,
    pub zone_name: String,
    pub zone_type: ZoneType,
    /// Index for proper numerical sorting (Episode 1, 2, 3... not 1, 10, 11, 2...)
    pub zone_index: i32,
    pub total_stages: i32,
    pub completed_stages: i32,
    pub perfect_stages: i32, // state >= 2 (3-star clears)
    pub completion_percentage: f32,
    pub score: f32,
}

/// Overall stage completion score (internal use)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageScore {
    pub total_score: f32,
    pub zone_scores: Vec<ZoneScore>,
    pub breakdown: StageBreakdown,
}

/// Breakdown of stage completion stats (internal use)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageBreakdown {
    pub mainline_completion: f32,  // Main story %
    pub sidestory_completion: f32, // Side stories %
    pub activity_completion: f32,  // Events %
    /// Permanent stages completed (mainline + sidestory only - always available content)
    pub permanent_stages_completed: i32,
    /// Permanent stages available (mainline + sidestory only)
    pub permanent_stages_available: i32,
    /// All stages completed (including time-limited activities)
    pub total_stages_completed: i32,
    /// All stages available (including time-limited activities)
    pub total_stages_available: i32,
    pub total_perfect_clears: i32,
}
