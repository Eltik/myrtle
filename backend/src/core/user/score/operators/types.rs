use serde::{Deserialize, Serialize};

/// Completion status of an operator based on investment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CompletionStatus {
    /// E0, no masteries, no modules
    NotStarted,
    /// Some investment but no M3
    InProgress,
    /// At least one M3 skill
    PartiallyCompleted,
    /// M6 or multiple modules at level 3
    HighlyInvested,
    /// M9 (all skills M3)
    AbsolutelyCompleted,
}

/// Score breakdown for a single operator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorScore {
    pub char_id: String,
    pub name: String,
    pub rarity: i32,
    pub base_score: f32,
    pub level_score: f32,
    pub trust_score: f32,
    pub potential_score: f32,
    pub mastery_score: f32,
    pub module_score: f32,
    pub skin_score: f32,
    pub total_score: f32,
    pub completion_status: CompletionStatus,
    pub mastery_details: MasteryDetails,
    pub module_details: ModuleDetails,
    pub skin_details: SkinDetails,
}

/// Details about mastery investment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasteryDetails {
    /// Number of skills at M3
    pub m3_count: i32,
    /// Total mastery levels across all skills (0-9)
    pub total_mastery_levels: i32,
}

/// Details about module investment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleDetails {
    /// Number of modules unlocked
    pub modules_unlocked: i32,
    /// Number of modules at level 3
    pub modules_at_max: i32,
    /// Highest module level
    pub highest_level: i32,
}

/// Details about skin collection for an operator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinDetails {
    /// Total number of skins owned (excluding default E0/E1/E2)
    pub owned_count: i32,
    /// Total number of skins available for this operator
    pub total_available: i32,
    /// Number of L2D (animated) skins owned
    pub owned_l2d: i32,
    /// Number of store-purchased skins owned
    pub owned_store: i32,
    /// Number of event reward skins owned
    pub owned_event: i32,
    /// Total L2D skins available
    pub total_l2d: i32,
    /// Total store skins available
    pub total_store: i32,
    /// Total event skins available
    pub total_event: i32,
    /// Collection completion percentage (0-100)
    pub completion_percentage: f32,
}
