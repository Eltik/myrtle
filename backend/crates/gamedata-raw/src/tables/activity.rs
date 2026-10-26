//! Events and missions: `activity_table` (parsed twice at boot today),
//! `open_server_table`, `mission_table`, mission archives.
//!
//! Moves in: `backend/src/core/gamedata/types/{activity.rs (469),
//! open_server.rs (54), mission.rs (213), mission_archive.rs (124)}`.

#[derive(Debug, Default)]
pub struct ActivityTables {}

// ---- Lifted for engine-release (lift pilot, Oct 25 2026). Copied from backend/src/core/gamedata/types/activity.rs:11-65,
// ts-rs dropped, utoipa behind the `schema` feature.
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

    /// When event-medal rewards stop being claimable (often a few days after
    /// `end_time`); 0 if absent. Authoritative close time for event medals.
    #[serde(alias = "RewardEndTime", default)]
    pub reward_end_time: i64,

    /// Group id tying this activity's medals together (e.g.
    /// `medalGroupActivity46side`). Empty when the activity has no medals.
    #[serde(alias = "MedalGroupId", default)]
    pub medal_group_id: String,

    #[serde(alias = "HasStage", default)]
    pub has_stage: bool,

    #[serde(alias = "IsReplicate", default)]
    pub is_replicate: bool,

    /// Activity category (`TYPE_ACT46SIDE`, `MULTIPLAY_V3`, `BOSS_RUSH`). One-time
    /// competitive events (CC, Boss Rush, Vector Breakthrough) can't be cleared
    /// after they end, so the stage universe drops them by this.
    #[serde(alias = "Type_", rename = "type", default)]
    pub activity_type: String,

    /// The Archives shelf the game files the event under: `SIDESTORY`,
    /// `BRANCHLINE` (Intermezzi), `MINISTORY` (Vignettes) or `NONE`. On EN
    /// 3 of the 50 `ACTIVITY_STORY` groups are `BRANCHLINE` (act9d0, act18d0,
    /// act18d3); the story reader shows it beside the derived category.
    #[serde(alias = "DisplayType", default)]
    pub display_type: String,

    /// The event token shop's id on the game server (`shop_act54side`), the
    /// argument to `templateShop/getGoodList`. Absent when the activity has no
    /// token shop (sign-ins, logins, and the few events with a bespoke shop).
    #[serde(alias = "TemplateShopId", default)]
    pub template_shop_id: Option<String>,
}

// ---- Lifted for engine-grade (lift-3, Oct 25 2026): the impl block from
// backend/src/core/gamedata/types/activity.rs:67-97 (medal scoring calls it).
impl ActivityBasicInfo {
    /// One-time competitive / minigame events (Contingency Contract / multiplayer,
    /// Vector Breakthrough, Boss Rush, Enemy Duel, Auto-Chess, Half-Idle, Arcade,
    /// ...) that can't be replayed after they end and aren't rebroadcast.
    ///
    /// Shared by the stage universe (drops their stages) and medal scoring (marks
    /// their medals unobtainable instead of recency-decaying them), so nobody is
    /// penalized for a one-off mode they never got to play.
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
