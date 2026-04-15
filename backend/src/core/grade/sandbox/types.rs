#[derive(Debug, Clone, Default)]
pub struct SandboxGradeDetail {
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
    pub tech_unlocked: usize,
    pub tech_total: usize,
    pub quests_completed: usize,
    pub quests_total: usize,
}
