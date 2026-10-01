//! `activity_table.AutoChessData.BondInfoDict`: the bonds of Stronghold
//! Protocol, the auto-chess mode.
//!
//! EN and CN carry the same 23 on 2026-09-05: 8 `SEASON` bonds (`IsPower`,
//! each naming the `handbook_team_table` factions it gathers in
//! `PowerIdList`) and 15 `REGULAR` trait bonds. `BandDataDict` (the squads) is
//! a separate table and is not read.
//!
//! The table is read from `activity_table.json` a second time with only this
//! field kept, the same way [`super::mission_archive`] reads its shelf:
//! `ActivityTableFile` belongs to the event code.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::core::gamedata::tables::SanitizingReader;

use super::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Deserialize)]
struct ActivityAutoChessFile {
    #[serde(alias = "AutoChessData", alias = "autoChessData", default)]
    auto_chess_data: AutoChessData,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AutoChessData {
    #[serde(
        alias = "BondInfoDict",
        alias = "bondInfoDict",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    bond_info_dict: HashMap<String, AutoChessBond>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AutoChessBond {
    #[serde(alias = "BondId", alias = "bondId", default)]
    pub bond_id: String,
    #[serde(alias = "Name", alias = "name", default)]
    pub name: String,
    /// `SEASON` (a faction bond) or `REGULAR` (a trait bond).
    #[serde(alias = "BondType", alias = "bondType", default)]
    pub bond_type: String,
    #[serde(alias = "BondOrder", alias = "bondOrder", default)]
    pub bond_order: i32,
    /// The PNG stem under `[uc]autochesscommon` (`icon_yanShip`).
    #[serde(alias = "Icon", alias = "icon", default)]
    pub icon: String,
    /// `handbook_team_table` ids a `SEASON` bond gathers.
    #[serde(alias = "PowerIdList", alias = "powerIdList", default)]
    pub power_id_list: Vec<String>,
}

/// Read the bonds out of `activity_table.json`. A missing or malformed file is
/// a WARNING and an empty map, by the rule `load_table_or_warn` follows.
pub fn load_autochess_bonds(
    data_dir: &Path,
    warnings: &mut Vec<String>,
) -> HashMap<String, AutoChessBond> {
    let path = data_dir.join("activity_table.json");
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            warnings.push(format!("autochess_bonds: {e}"));
            return HashMap::new();
        }
    };
    let reader = SanitizingReader::new(std::io::BufReader::new(file));
    match serde_json::from_reader::<_, ActivityAutoChessFile>(reader) {
        Ok(t) => t.auto_chess_data.bond_info_dict,
        Err(e) => {
            warnings.push(format!("autochess_bonds: {e}"));
            HashMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bond_reads_its_type_icon_and_factions() {
        let raw = r#"{"BasicInfo": [], "AutoChessData": {"BandDataDict": [], "BondInfoDict": [
            {"key": "yanShip", "value": {"BondId": "yanShip", "BondOrder": 2,
             "BondType": "SEASON", "Icon": "icon_yanShip", "IsHiddenCharList": false,
             "IsPower": true, "Name": "Yan", "PowerIdList": ["yan", "sui"]}}]}}"#;
        let file: ActivityAutoChessFile = serde_json::from_str(raw).expect("parses");
        let b = &file.auto_chess_data.bond_info_dict["yanShip"];
        assert_eq!(b.name, "Yan");
        assert_eq!(b.bond_type, "SEASON");
        assert_eq!(b.icon, "icon_yanShip");
        assert_eq!(b.power_id_list, ["yan", "sui"]);
    }
}
