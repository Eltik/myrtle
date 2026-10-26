//! The game's own Archives network: `handbook_force_map_table` (the
//! territories map, one hex cluster per force) and `handbookpos_table` (one
//! chart per group, every member a hex cell). Both ship under
//! `gamedata/art/` as `*.json.json`, camelCase, and no other code reads them.
//!
//! Coordinates are the game's, Unity style with y up. Territory hexes are
//! flat-topped with circumradius 50 (neighbours 86.6 apart), chart cells
//! circumradius 100 (neighbours 173.2 apart). A connection's `direction`
//! names a neighbour: 1 is straight up, then clockwise in 60 degree steps, so
//! direction d points at `150 - 60 d` degrees (measured on Rhodes Island's
//! seven-hex cluster: d 1 -> (0, 86.6), d 2 -> (75, 43.3), d 0 -> (-75, 43.3)).
//!
//! Lifted by lift-2 (Oct 25 2026) for engine-archives: the serde structs of
//! backend/src/core/archives/network.rs:20-185, moved down because they are
//! table shapes (files.csv row `core/archives/network.rs`: "`std::fs::read` of
//! `handbook_force_map_table` at L197 -> gamedata-raw table"). The file read
//! (`load_network`, `read_table`, network.rs:187-201) is the caller's; this
//! module parses bytes the caller hands it.

use std::collections::HashMap;

use serde::Deserialize;

use crate::serde_helpers::deserialize_null_default;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Vec2 {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceMapTable {
    #[serde(default)]
    pub force_line_list: Vec<ForceLine>,
    #[serde(default)]
    pub force_list: Vec<RawForce>,
    #[serde(default)]
    pub point_line_list: Vec<PointLine>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceLine {
    pub force_id1: String,
    pub force_id2: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointLine {
    pub from_idx: u32,
    pub to_idx: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawForce {
    pub force_id: String,
    #[serde(default)]
    pub force_index: i32,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub color: String,
    /// `null` on 32 of EN's 41 forces. Every `deserialize_null_default`
    /// string here reads a `null` as empty.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub card_color: String,
    #[serde(default)]
    pub card_pos: Vec2,
    #[serde(default)]
    pub card_scale: f64,
    #[serde(default)]
    pub logo_pos: Vec2,
    #[serde(default)]
    pub logo_scale: f64,
    #[serde(default)]
    pub point_list: Vec<RawPoint>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPoint {
    pub point_index: u32,
    pub pos: Vec2,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosTable {
    #[serde(default)]
    pub force_to_group_dict: HashMap<String, String>,
    #[serde(default)]
    pub group_list: HashMap<String, RawGroup>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawGroup {
    pub group_id: String,
    #[serde(default)]
    pub char_list: Vec<RawCell>,
    #[serde(default)]
    pub force_data_list: Vec<RawBlock>,
    #[serde(default)]
    pub line_list: Vec<RawLine>,
    #[serde(default)]
    pub min_scale: f64,
    #[serde(default)]
    pub max_scale: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawConnection {
    pub char_id: Option<String>,
    pub direction: u8,
    pub force_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawCell {
    pub char_id: String,
    #[serde(default)]
    pub connection_list: Vec<RawConnection>,
    #[serde(default)]
    pub have_out_line: bool,
    pub pos: Vec2,
    #[serde(default = "one")]
    pub scale: f64,
}

const fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawBlock {
    #[serde(default)]
    pub char_list: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub color: String,
    #[serde(default)]
    pub color_blocks: Vec<RawColorBlock>,
    #[serde(default)]
    pub connect_pos: i32,
    #[serde(default)]
    pub focus_pos: Vec2,
    pub force_id: String,
    #[serde(default)]
    pub is_this_group: bool,
    #[serde(default)]
    pub pos: Vec2,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawColorBlock {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub color: String,
    #[serde(default)]
    pub connection_list: Vec<RawConnection>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub force_id: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub id: String,
    pub pos: Vec2,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawLine {
    pub char1: String,
    pub char2: String,
    #[serde(default)]
    pub line_type: i32,
}

/// Both tables, read together: one without the other draws nothing.
#[derive(Debug, Clone, Default)]
pub struct RawNetwork {
    pub force_map: ForceMapTable,
    pub positions: PosTable,
}

impl RawNetwork {
    /// Parse both tables (`handbook_force_map_table.json.json`,
    /// `handbookpos_table.json.json`) from bytes. Replaces the parse half of
    /// `read_table` (network.rs:196-201); on failure the error names the table
    /// so the caller can log the same "{name} did not parse" warning.
    pub fn from_slices(
        force_map: &[u8],
        positions: &[u8],
    ) -> Result<Self, (&'static str, serde_json::Error)> {
        Ok(Self {
            force_map: serde_json::from_slice(force_map)
                .map_err(|e| ("handbook_force_map_table", e))?,
            positions: serde_json::from_slice(positions).map_err(|e| ("handbookpos_table", e))?,
        })
    }
}
