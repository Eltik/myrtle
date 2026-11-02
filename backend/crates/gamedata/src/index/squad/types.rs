use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyCond {
    Squad,
    Field,
    Skill,
    Near,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyTargetKind {
    Everyone,
    Class,
    Faction,
    Tag,
    SpType,
    Position,
    CostAtMost,
    Next,
    Unknown,
}
