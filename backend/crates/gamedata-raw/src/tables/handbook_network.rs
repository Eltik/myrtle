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
    /// Neighbour direction: 1 = straight up, then clockwise in 60 degree steps
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

/// `handbook_force_map_table` + `handbookpos_table`
#[derive(Debug, Clone, Default)]
pub struct RawNetwork {
    pub force_map: ForceMapTable,
    pub positions: PosTable,
}

impl RawNetwork {
    /// Parse both tables from bytes; the error names the failing table
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
