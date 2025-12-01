use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::material::ItemType;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorGender {
    Unknown,
    Female,
    Male,
    #[serde(rename = "Male]")]
    MaleBugged, // Arene is bugged and has ] at the end
    Conviction,
}

impl Default for OperatorGender {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorBirthPlace {
    Unknown,
    Undisclosed,
    Higashi,
    Kazimierz,
    Vouivre,
    Laterano,
    Victoria,
    #[serde(rename = "Rim Billiton")]
    RimBilliton,
    Leithanien,
    #[serde(rename = "Bolívar")]
    Bolivar,
    Sargon,
    Kjerag,
    Columbia,
    Sami,
    Iberia,
    Kazdel,
    Minos,
    Lungmen,
    Siracusa,
    Yan,
    Ursus,
    Siesta,
    #[serde(rename = "RIM Billiton")]
    RIMBilliton,
    #[serde(rename = "Ægir")]
    Aegir,
    Durin,
    #[serde(rename = "Siesta (Independent City)")]
    SiestaIndependentCity,
    #[serde(rename = "Ægir Region")]
    AegirRegion,
    #[serde(rename = "Unknown as requested by management agency")]
    UnknownAsRequestedByManagementAgency,
    #[serde(rename = "Rhodes Island")]
    RhodesIsland,
    #[serde(rename = "Far East")]
    FarEast,
}

impl Default for OperatorBirthPlace {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorRace {
    Undisclosed,
    Zalak,
    Oni,
    Savra,
    Durin,
    Kuranta,
    Vouivre,
    Liberi,
    Feline,
    Cautus,
    Perro,
    Reproba,
    Sankta,
    Sarkaz,
    Vulpo,
    Elafia,
    Phidia,
    #[serde(rename = "Ægir")]
    Aegir,
    Anaty,
    Itra,
    #[serde(rename = "Unknown (Suspected Liberi)")]
    UnknownSuspectedLiberi,
    Archosauria,
    Unknown,
    Lupo,
    Forte,
    Ursus,
    Petram,
    Cerato,
    Caprinae,
    Draco,
    Anura,
    Anasa,
    #[serde(rename = "Cautus/Chimera")]
    CautusChimera,
    Kylin,
    Pilosa,
    #[serde(rename = "Unknown as requested by management agency")]
    UnknownAsRequestedByManagementAgency,
    Manticore,
    Lung,
    Aslan,
    Elf,
    #[serde(rename = "Sa■&K?uSxw?")]
    Corrupted, // Special corrupted text for certain operators
}

impl Default for OperatorRace {
    fn default() -> Self {
        Self::Unknown
    }
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicInfo {
    pub code_name: String,
    pub gender: OperatorGender,
    pub combat_experience: String,
    pub place_of_birth: OperatorBirthPlace,
    pub date_of_birth: String,
    pub race: OperatorRace,
    pub height: String,
    pub infection_status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalExam {
    pub physical_strength: String,
    pub mobility: String,
    pub physical_resilience: String,
    pub tactical_acumen: String,
    pub combat_skill: String,
    pub originium_arts_assimilation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorProfile {
    pub basic_info: BasicInfo,
    pub physical_exam: PhysicalExam,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookRewardItem {
    pub id: String,
    pub count: i32,
    #[serde(rename = "type")]
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMission {
    pub id: String,
    pub sort: i32,
    pub power_id: String,
    pub power_name: String,
    pub item: HandbookRewardItem,
    pub favor_point: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookDisplayCondition {
    pub char_id: String,
    pub condition_char_id: String,
    #[serde(rename = "type")]
    pub condition_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStageTime {
    pub timestamp: i64,
    pub char_set: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStory {
    pub story_text: String,
    pub unlock_type: String,
    pub un_lock_param: String,
    pub un_lock_string: String,
    pub patch_id_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStoryTextAudio {
    pub stories: Vec<HandbookStory>,
    pub story_title: String,
    pub un_lockor_not: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookUnlockParam {
    pub unlock_type: String,
    pub unlock_param1: Option<String>,
    pub unlock_param2: Option<String>,
    pub unlock_param3: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookAvgEntry {
    pub story_id: String,
    pub story_set_id: String,
    pub story_sort: i32,
    pub story_can_show: bool,
    pub story_intro: String,
    pub story_info: String,
    pub story_txt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookAvgList {
    pub story_set_id: String,
    pub story_set_name: String,
    pub sort_id: i32,
    pub story_get_time: i64,
    pub reward_item: Vec<HandbookRewardItem>,
    pub unlock_param: Vec<HandbookUnlockParam>,
    pub avg_list: Vec<HandbookAvgEntry>,
    pub char_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookItem {
    #[serde(rename = "charID")]
    pub char_id: String,
    pub info_name: String,
    pub is_limited: bool,
    pub story_text_audio: Vec<HandbookStoryTextAudio>,
    pub handbook_avg_list: Vec<HandbookAvgList>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NPCUnlockInfo {
    pub un_lock_type: String,
    pub un_lock_param: String,
    pub un_lock_string: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookNPCItem {
    pub npc_id: String,
    pub name: String,
    pub appellation: String,
    pub profession: String,
    pub illust_list: Option<Vec<String>>,
    pub designer_list: Option<Vec<String>>,
    pub cv: String,
    pub display_number: String,
    pub nation_id: Option<String>,
    pub group_id: Option<String>,
    pub team_id: Option<String>,
    pub res_type: String,
    pub npc_show_audio_info_flag: bool,
    pub unlock_dict: HashMap<String, NPCUnlockInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStageData {
    pub char_id: String,
    pub stage_id: String,
    pub level_id: String,
    pub zone_id: String,
    pub code: String,
    pub name: String,
    pub loading_pic_id: String,
    pub description: String,
    pub unlock_param: Vec<HandbookUnlockParam>,
    pub reward_item: Vec<HandbookRewardItem>,
    pub stage_name_for_show: String,
    pub zone_name_for_show: String,
    pub pic_id: String,
    pub stage_get_time: i64,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handbook {
    pub handbook_dict: HashMap<String, HandbookItem>,
    pub npc_dict: HashMap<String, HandbookNPCItem>,
    pub team_mission_list: HashMap<String, TeamMission>,
    pub handbook_display_condition_list: HashMap<String, HandbookDisplayCondition>,
    pub handbook_stage_data: HashMap<String, HandbookStageData>,
    pub handbook_stage_time: Vec<HandbookStageTime>,
}
