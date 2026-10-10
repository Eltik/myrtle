//! The returning-player ("Doctor's return") program in `open_server_table`,
//! read for one fact: which gacha pools are shown only to returning players.
//!
//! `PlayerReturn.GroupDataMap` holds one group per return tier
//! (`return_30_g_1`, `return_180_g_1`, `return_old_g_1`, and on CN further
//! `_g_2`/`_g_3`/`returnGroup_*` tiers). A tier that grants a personal pool
//! names it in `GachaPoolId`. That pool is the BACKFLOW "Returning Headhunting"
//! banner, whose `gacha_table` window runs 1,513 to 1,523 days because the game
//! times it per player, not by calendar; its summary is the runtime template
//! "Ends at {0}". Measured 2026-10-10 on en, cn, jp and kr: one pool per server
//! (`RETURN_EN_41_0_1`, `RETURN_71_0_1`, `RETURN_JP_41_0_1`,
//! `RETURN_KR_41_0_1`), linked only from the 180-day and older tiers, and the
//! set matches the BACKFLOW rule type 1:1.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use super::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OpenServerTableFile {
    #[serde(default)]
    pub player_return: PlayerReturn,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerReturn {
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub group_data_map: HashMap<String, ReturnGroup>,
}

/// One return tier. Only the pool it grants is kept.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReturnGroup {
    #[serde(default)]
    pub gacha_pool_id: Option<String>,
}

impl OpenServerTableFile {
    /// Every gacha pool some return tier grants.
    pub fn returning_pool_ids(&self) -> HashSet<String> {
        self.player_return
            .group_data_map
            .values()
            .filter_map(|g| g.gacha_pool_id.as_deref())
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
            .collect()
    }
}
