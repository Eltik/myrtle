use std::collections::HashMap;

use serde::Deserialize;

use crate::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Default)]
pub struct AutochessTables {
    /// `activity_table`, `AutoChessData` only
    pub autochess: ActivityAutoChessFile,
}

/// `activity_table.AutoChessData.BondInfoDict`, Stronghold Protocol bonds
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActivityAutoChessFile {
    #[serde(alias = "AutoChessData", alias = "autoChessData", default)]
    pub auto_chess_data: AutoChessData,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AutoChessData {
    #[serde(
        alias = "BondInfoDict",
        alias = "bondInfoDict",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub bond_info_dict: HashMap<String, AutoChessBond>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AutoChessBond {
    #[serde(alias = "BondId", alias = "bondId", default)]
    pub bond_id: String,
    #[serde(alias = "Name", alias = "name", default)]
    pub name: String,
    /// `SEASON` (a faction bond) or `REGULAR` (a trait bond)
    #[serde(alias = "BondType", alias = "bondType", default)]
    pub bond_type: String,
    #[serde(alias = "BondOrder", alias = "bondOrder", default)]
    pub bond_order: i32,
    /// PNG stem under `[uc]autochesscommon` (`icon_yanShip`)
    #[serde(alias = "Icon", alias = "icon", default)]
    pub icon: String,
    /// `handbook_team_table` ids a `SEASON` bond gathers
    #[serde(alias = "PowerIdList", alias = "powerIdList", default)]
    pub power_id_list: Vec<String>,
}
