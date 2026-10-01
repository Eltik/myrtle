//! `handbook_team_table`: the game's factions. One id space for three levels,
//! the nations (`PowerLevel` 0), the groups inside them (1) and the teams (2).
//!
//! An operator's `NationId`, `GroupId` and `TeamId` are keys here. On the EN
//! table of 2026-09-05 the 46 keys are 20 nations, 15 groups and 11 teams;
//! `none` is the sentinel an operator without a faction points at, and it is
//! not a faction.

use std::collections::HashMap;

use serde::Deserialize;

use super::serde_helpers::deserialize_fb_map_or_default;

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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HandbookTeam {
    #[serde(alias = "PowerId", alias = "powerId", default)]
    pub power_id: String,
    #[serde(alias = "PowerName", alias = "powerName", default)]
    pub power_name: String,
    #[serde(alias = "PowerCode", alias = "powerCode", default)]
    pub power_code: String,
    /// 0 nation, 1 group, 2 team.
    #[serde(alias = "PowerLevel", alias = "powerLevel", default)]
    pub power_level: i32,
    /// The handbook's own order, nations first.
    #[serde(alias = "OrderNum", alias = "orderNum", default)]
    pub order_num: i32,
    /// Hex without `#`. `000000` on every EN row of 2026-09-05.
    #[serde(alias = "Color", alias = "color", default)]
    pub color: String,
}

/// The sentinel id an operator with no nation, group or team carries.
pub const NO_FACTION: &str = "none";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_team_row_reads_its_level_and_name() {
        let raw = r#"{"Handbook_teams": [{"key": "rhodes", "value": {
            "Color": "000000", "IsLimited": false, "IsRaw": false, "OrderNum": 1,
            "PowerCode": "Rhodes Island", "PowerId": "rhodes", "PowerLevel": 0,
            "PowerName": "Rhodes Island"}}]}"#;
        let file: HandbookTeamTableFile = serde_json::from_str(raw).expect("parses");
        let t = &file.teams["rhodes"];
        assert_eq!(t.power_name, "Rhodes Island");
        assert_eq!(t.power_level, 0);
        assert_eq!(t.order_num, 1);
    }
}
