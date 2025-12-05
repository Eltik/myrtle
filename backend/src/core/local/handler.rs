use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::core::local::gamedata::operators::enrich_all_operators;
use crate::core::local::gamedata::skills::enrich_all_skills;
use crate::core::local::types::handbook::{Handbook, HandbookTableFile};
use crate::core::local::types::material::{ItemTableFile, Materials};
use crate::core::local::types::module::{
    BattleEquip, BattleEquipTableFile, Modules, RawModules, UniequipTableFile,
};
use crate::core::local::types::skill::{RawSkill, SkillTableFile};
use crate::core::local::types::skin::{SkinData, SkinTableFile};
use crate::core::local::types::{GameData, operator::CharacterTable};

#[derive(Debug)]
pub enum DataError {
    Io(std::io::Error),
    Parse {
        table: String,
        error: serde_json::Error,
    },
    Missing {
        table: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(e) => write!(f, "IO error: {}", e),
            DataError::Parse { table, error } => write!(f, "Failed to parse {}: {}", table, error),
            DataError::Missing { table } => write!(f, "Missing table file: {}", table),
        }
    }
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

impl std::error::Error for DataError {}

pub struct DataHandler {
    data_dir: PathBuf,
}

impl DataHandler {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn load_table<T: DeserializeOwned>(&self, table_name: &str) -> Result<T, DataError> {
        let path = self.data_dir.join(format!("{}.json", table_name));
        let file = std::fs::File::open(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DataError::Missing {
                    table: table_name.to_string(),
                }
            } else {
                DataError::Io(e)
            }
        })?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| DataError::Parse {
            table: table_name.to_string(),
            error: e,
        })
    }

    pub fn load_table_or_default<T: DeserializeOwned + Default>(&self, table_name: &str) -> T {
        self.load_table(table_name).unwrap_or_default()
    }
}

pub fn init_game_data(data_dir: &Path) -> Result<GameData, DataError> {
    let handler = DataHandler::new(data_dir);

    // ============ Load Character Table ============
    let character_table = handler.load_table::<CharacterTable>("character_table")?;
    let raw_operators = character_table.characters;

    // ============ Load Skill Table ============
    let raw_skills: HashMap<String, RawSkill> =
        match handler.load_table::<SkillTableFile>("skill_table") {
            Ok(skill_table) => skill_table.skills,
            Err(e) => {
                eprintln!("Warning: Failed to load skill_table: {}", e);
                HashMap::new()
            }
        };

    // ============ Load Uniequip (Module) Table ============
    let raw_modules: RawModules = match handler.load_table::<UniequipTableFile>("uniequip_table") {
        Ok(uniequip_table) => RawModules {
            equip_dict: uniequip_table.equip_dict,
            mission_list: uniequip_table.mission_list,
            sub_prof_dict: uniequip_table.sub_prof_dict,
            char_equip: uniequip_table.char_equip,
            equip_track_dict: uniequip_table.equip_track_dict,
        },
        Err(e) => {
            eprintln!("Warning: Failed to load uniequip_table: {}", e);
            RawModules::default()
        }
    };

    // ============ Load Battle Equip Table ============
    let battle_equip: BattleEquip =
        match handler.load_table::<BattleEquipTableFile>("battle_equip_table") {
            Ok(battle_equip_table) => battle_equip_table.equips,
            Err(e) => {
                eprintln!("Warning: Failed to load battle_equip_table: {}", e);
                HashMap::new()
            }
        };

    // ============ Load Handbook Table ============
    let handbook: Handbook = match handler.load_table::<HandbookTableFile>("handbook_info_table") {
        Ok(handbook_table) => Handbook {
            handbook_dict: handbook_table.handbook_dict,
            npc_dict: handbook_table.npc_dict,
            team_mission_list: handbook_table.team_mission_list,
            handbook_display_condition_list: handbook_table.handbook_display_condition_list,
            handbook_stage_data: handbook_table.handbook_stage_data,
            handbook_stage_time: handbook_table.handbook_stage_time,
        },
        Err(e) => {
            eprintln!("Warning: Failed to load handbook_info_table: {}", e);
            Handbook::default()
        }
    };

    // ============ Load Skin Table ============
    let skins: SkinData = match handler.load_table::<SkinTableFile>("skin_table") {
        Ok(skin_table) => SkinData {
            char_skins: skin_table.char_skins,
            buildin_evolve_map: HashMap::new(), // These have complex nested structures
            buildin_patch_map: HashMap::new(),
            brand_list: skin_table.brand_list,
            special_skin_info_list: skin_table.special_skin_info_list,
        },
        Err(e) => {
            eprintln!("Warning: Failed to load skin_table: {}", e);
            SkinData::default()
        }
    };

    // ============ Load Item Table ============
    let materials: Materials = match handler.load_table::<ItemTableFile>("item_table") {
        Ok(item_table) => Materials {
            items: item_table.items,
            exp_items: item_table.exp_items,
            potential_items: HashMap::new(), // Complex nested structure
            ap_supplies: item_table.ap_supply_out_of_date_dict,
            char_voucher_items: item_table.char_voucher_items,
        },
        Err(e) => {
            eprintln!("Warning: Failed to load item_table: {}", e);
            Materials::default()
        }
    };

    // ============ Enrich Data ============
    let skills = enrich_all_skills(raw_skills);

    let operators = enrich_all_operators(
        &raw_operators,
        &skills,
        &raw_modules,
        &battle_equip,
        &handbook,
        &skins,
    );

    // Convert raw modules to enriched modules
    let modules = Modules {
        equip_dict: raw_modules
            .equip_dict
            .into_iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    crate::core::local::types::module::Module {
                        id: Some(k),
                        uni_equip_id: v.uni_equip_id,
                        uni_equip_name: v.uni_equip_name,
                        uni_equip_icon: v.uni_equip_icon,
                        image: None,
                        uni_equip_desc: v.uni_equip_desc,
                        type_icon: v.type_icon,
                        type_name1: v.type_name1,
                        type_name2: v.type_name2,
                        equip_shining_color: v.equip_shining_color,
                        show_evolve_phase: v.show_evolve_phase,
                        unlock_evolve_phase: v.unlock_evolve_phase,
                        char_id: v.char_id,
                        tmpl_id: v.tmpl_id,
                        show_level: v.show_level,
                        unlock_level: v.unlock_level,
                        unlock_favor_point: v.unlock_favor_point,
                        mission_list: v.mission_list,
                        item_cost: None,
                        module_type: v.module_type,
                        uni_equip_get_time: v.uni_equip_get_time,
                        char_equip_order: v.char_equip_order,
                    },
                )
            })
            .collect(),
        mission_list: raw_modules.mission_list,
        sub_prof_dict: raw_modules.sub_prof_dict,
        char_equip: raw_modules.char_equip,
        equip_track_dict: std::collections::HashMap::new(),
        battle_equip,
    };

    Ok(GameData {
        operators,
        skills,
        materials,
        modules,
        skins,
        handbook,
    })
}

pub fn init_game_data_or_default(data_dir: &Path) -> GameData {
    match init_game_data(data_dir) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Warning: Failed to load game data: {}", e);
            eprintln!("Starting with empty game data");
            GameData::default()
        }
    }
}
