/// The RA grade: `total` is the equal-weight mean of `topics`' totals.
#[derive(Debug, Clone, Default)]
pub struct SandboxGrade {
    pub total: f64,
    /// One entry per graded topic, in `BasicInfo` `SortId` order.
    pub topics: Vec<SandboxGradeDetail>,
}

#[derive(Debug, Clone, Default)]
pub struct SandboxGradeDetail {
    /// The topic graded (`sandbox_1`, `sandbox_2`).
    pub topic_id: Option<String>,
    /// `BasicInfo.TopicName` of the topic; empty when the table has none.
    pub topic_name: String,
    /// Graded on the `SANDBOX_V3` reader: bands instead of blueprints, no rifts.
    pub v3: bool,
    pub total: f64,
    pub achievements: f64,
    pub exploration: f64,
    pub tech_tree: f64,
    pub quests: f64,
    pub base_building: f64,
    pub content_depth: f64,
    pub achievements_completed: usize,
    pub achievements_total: usize,
    pub nodes_explored: usize,
    pub nodes_total: usize,
    pub zones_unlocked: usize,
    pub zones_total: usize,
    pub tech_unlocked: usize,
    pub tech_total: usize,
    pub quests_completed: usize,
    pub quests_total: usize,
    pub base_level: usize,
    pub base_level_max: usize,
    pub blueprints: usize,
    pub blueprints_total: usize,
    pub recipes: usize,
    pub recipes_total: usize,
    pub music: usize,
    pub music_total: usize,
    /// Summed rift difficulty levels cleared (the numerator of the rift score).
    pub rift_levels: usize,
    /// Max possible summed rift difficulty (`rifts × 16`); the rift-score denominator.
    pub rift_levels_max: usize,
    /// Summed band levels (V3 only), and their maximum.
    pub band_levels: usize,
    pub band_levels_max: usize,
}
