use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::core::local::asset_mapping::AssetMappings;
use crate::core::local::gamedata::operators::enrich_all_operators;
use crate::core::local::gamedata::skills::enrich_all_skills;
use crate::core::local::gamedata::skins::enrich_all_skins;
use crate::core::local::gamedata::voice::enrich_all_voices;
use crate::core::local::types::gacha::{GachaData, GachaTableFile};
use crate::core::local::types::handbook::{Handbook, HandbookTableFile};
use crate::core::local::types::material::{ItemTableFile, Materials};
use crate::core::local::types::module::{
    BattleEquip, BattleEquipTableFile, Modules, RawModules, UniequipTableFile,
};
use crate::core::local::types::range::Ranges;
use crate::core::local::types::skill::{RawSkill, SkillTableFile};
use crate::core::local::types::skin::{SkinData, SkinTableFile};
use crate::core::local::types::trust::Favor;
use crate::core::local::types::voice::{Voices, VoicesTableFile};
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

pub fn init_game_data(data_dir: &Path, assets_dir: &Path) -> Result<GameData, DataError> {
    let handler = DataHandler::new(data_dir);

    // ============ Build Asset Mappings (early, needed for skins/modules) ============
    let asset_mappings = AssetMappings::build(assets_dir);

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
        Ok(skin_table) => {
            let enriched_skins = enrich_all_skins(&skin_table.char_skins, &asset_mappings);

            SkinData {
                char_skins: skin_table.char_skins,
                buildin_evolve_map: HashMap::new(),
                buildin_patch_map: HashMap::new(),
                brand_list: skin_table.brand_list,
                special_skin_info_list: skin_table.special_skin_info_list,
                enriched_skins,
            }
        }
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

    // ============ Load Range Table ============
    let ranges: Ranges = match handler.load_table::<Ranges>("range_table") {
        Ok(range_data) => range_data,
        Err(e) => {
            eprintln!("Warning: Failed to load range_table: {}", e);
            HashMap::new()
        }
    };

    // ============ Load Favor Table ============
    let favor: Favor = match handler.load_table::<Favor>("favor_table") {
        Ok(favor_data) => favor_data,
        Err(e) => {
            eprintln!("Warning: Failed to load favor_table: {}", e);
            Favor::default()
        }
    };

    // ============ Load Charword (Voice) Table ============
    let voices: Voices = match handler.load_table::<VoicesTableFile>("charword_table") {
        Ok(voice_table) => {
            let enriched_char_words =
                enrich_all_voices(&voice_table.char_words, &voice_table.voice_lang_dict);

            Voices {
                char_words: enriched_char_words,
                char_extra_words: voice_table.char_extra_words,
                voice_lang_dict: voice_table.voice_lang_dict,
                default_lang_type: voice_table.default_lang_type,
                // ... populate other fields or use defaults
                ..Default::default()
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to load charword_table: {}", e);
            Voices::default()
        }
    };

    // ============ Load Gacha Table ============
    let gacha: GachaData = match handler.load_table::<GachaTableFile>("gacha_table") {
        Ok(gacha_table) => GachaData {
            gacha_pool_client: gacha_table.gacha_pool_client,
            newbee_gacha_pool_client: gacha_table.newbee_gacha_pool_client,
            special_recruit_pool: gacha_table.special_recruit_pool,
            gacha_tags: gacha_table.gacha_tags,
            recruit_pool: gacha_table.recruit_pool,
            potential_material_converter: gacha_table.potential_material_converter,
            classic_potential_material_converter: gacha_table.classic_potential_material_converter,
            recruit_rarity_table: gacha_table.recruit_rarity_table,
            special_tag_rarity_table: gacha_table.special_tag_rarity_table,
            recruit_detail: gacha_table.recruit_detail,
            show_gacha_log_entry: gacha_table.show_gacha_log_entry,
            carousel: gacha_table.carousel,
            free_gacha: gacha_table.free_gacha,
            limit_ten_gacha_item: gacha_table.limit_ten_gacha_item,
            linkage_ten_gacha_item: gacha_table.linkage_ten_gacha_item,
            normal_gacha_item: gacha_table.normal_gacha_item,
            fes_gacha_pool_relate_item: gacha_table.fes_gacha_pool_relate_item,
            dic_recruit6_star_hint: gacha_table.dic_recruit6_star_hint,
            special_gacha_percent_dict: gacha_table.special_gacha_percent_dict,
        },
        Err(e) => {
            eprintln!("Warning: Failed to load gacha_table: {}", e);
            GachaData::default()
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
        &materials,
        &asset_mappings,
    );

    // Convert raw modules to enriched modules
    let modules = Modules {
        equip_dict: raw_modules
            .equip_dict
            .into_iter()
            .map(|(k, v)| {
                // Get module image path using asset mappings
                let image = Some(asset_mappings.get_module_big_path(&v.uni_equip_icon));
                (
                    k.clone(),
                    crate::core::local::types::module::Module {
                        id: Some(k),
                        uni_equip_id: v.uni_equip_id,
                        uni_equip_name: v.uni_equip_name,
                        uni_equip_icon: v.uni_equip_icon,
                        image,
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
        ranges,
        favor,
        voices,
        gacha,
        asset_mappings,
    })
}

pub fn init_game_data_or_default(data_dir: &Path, assets_dir: &Path) -> GameData {
    match init_game_data(data_dir, assets_dir) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Warning: Failed to load game data: {}", e);
            eprintln!("Starting with empty game data");
            GameData::default()
        }
    }
}
