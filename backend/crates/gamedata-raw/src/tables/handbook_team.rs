use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HandbookTeamTableFile {
    #[serde(
        alias = "Handbook_teams",
        alias = "handbookTeams",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub teams: HashMap<String, HandbookTeam>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandbookTeam {
    #[serde(alias = "PowerId", alias = "powerId", default)]
    pub power_id: String,
    #[serde(alias = "PowerName", alias = "powerName", default)]
    pub power_name: String,
    #[serde(alias = "PowerCode", alias = "powerCode", default)]
    pub power_code: String,
    /// 0 nation, 1 group, 2 team
    #[serde(alias = "PowerLevel", alias = "powerLevel", default)]
    pub power_level: i32,
    /// Handbook order, nations first
    #[serde(alias = "OrderNum", alias = "orderNum", default)]
    pub order_num: i32,
    /// Hex without `#`
    #[serde(alias = "Color", alias = "color", default)]
    pub color: String,
}

/// Sentinel id for an operator with no nation, group or team
pub const NO_FACTION: &str = "none";
