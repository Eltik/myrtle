use serde::Deserialize;
use serde_json::Value;

/// `crisis_v2_table` (Contingency Contract), season names only
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CrisisV2TableFile {
    /// `SeasonInfoDataMap`, untyped: an object or a `FlatBuffer` `[{key, value}]` list
    #[serde(rename = "SeasonInfoDataMap", default)]
    pub season_info_data_map: Value,
}
