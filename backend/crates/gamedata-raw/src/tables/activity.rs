#[derive(Debug, Default)]
pub struct ActivityTables {}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct ActivityBasicInfo {
    #[serde(alias = "Id")]
    pub id: String,

    #[serde(alias = "Name", default)]
    pub name: String,

    #[serde(alias = "StartTime", default)]
    pub start_time: i64,

    #[serde(alias = "EndTime", default)]
    pub end_time: i64,

    /// When event-medal rewards stop being claimable, 0 if absent
    #[serde(alias = "RewardEndTime", default)]
    pub reward_end_time: i64,

    /// Group id tying medals together (e.g.
    /// `medalGroupActivity46side`)
    #[serde(alias = "MedalGroupId", default)]
    pub medal_group_id: String,

    #[serde(alias = "HasStage", default)]
    pub has_stage: bool,

    #[serde(alias = "IsReplicate", default)]
    pub is_replicate: bool,

    /// Activity category (`TYPE_ACT46SIDE`, `MULTIPLAY_V3`, `BOSS_RUSH`)
    #[serde(alias = "Type_", rename = "type", default)]
    pub activity_type: String,

    /// Overall category, eg. `SIDESTORY`,
    /// `BRANCHLINE` (Intermezzi), `MINISTORY` (Vignettes) or `NONE`
    #[serde(alias = "DisplayType", default)]
    pub display_type: String,

    /// Event token shop's id on game server (`shop_act54side`), used in `templateShop/getGoodList`
    #[serde(alias = "TemplateShopId", default)]
    pub template_shop_id: Option<String>,
}

impl ActivityBasicInfo {
    pub fn is_one_time_competitive(&self) -> bool {
        matches!(
            self.activity_type.as_str(),
            "MULTIPLAY"
                | "MULTIPLAY_V3"
                | "MULTIPLAY_VERIFY2"
                | "VEC_BREAK"
                | "VEC_BREAK_V2"
                | "BOSS_RUSH"
                | "ENEMY_DUEL"
                | "HALFIDLE_VERIFY1"
                | "AUTOCHESS_VERIFY1"
                | "AUTOCHESS_SEASON"
                | "ARCADE"
                | "FLOAT_PARADE"
                | "TEAM_QUEST"
                | "COLLECTION"
                | "INTERLOCK"
                | "MAINLINE_BP"
                | "FIREWORK"
        )
    }
}
