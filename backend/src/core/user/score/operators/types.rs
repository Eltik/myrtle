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

/// Summary statistics for the account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreBreakdown {
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
}

/// Total account score with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserScore {
    pub total_score: f32,
    pub operator_scores: Vec<OperatorScore>,
    pub breakdown: ScoreBreakdown,
}

impl Default for ScoreBreakdown {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Default for UserScore {
    fn default() -> Self {
        Self {
            total_score: 0.0,
            operator_scores: Vec::new(),
            breakdown: ScoreBreakdown::default(),
        }
    }
}
