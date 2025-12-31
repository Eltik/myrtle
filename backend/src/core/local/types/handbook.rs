use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::material::ItemType;
use super::serde_helpers::{deserialize_fb_map, deserialize_fb_map_or_default};

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OperatorGender {
    #[default]
    Unknown,
    Female,
    Male,
    #[serde(rename = "Male]")]
    MaleBugged, // Arene is bugged and has ] at the end
    Conviction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OperatorBirthPlace {
    #[default]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    #[default]
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
    #[serde(rename = "Sa■&K?uSxw?")]
    Corrupted, // Special corrupted text for certain operators
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicInfo {
    #[serde(alias = "CodeName")]
    pub code_name: String,
    #[serde(alias = "Gender")]
    pub gender: OperatorGender,
    #[serde(alias = "CombatExperience")]
    pub combat_experience: String,
    #[serde(alias = "PlaceOfBirth")]
    pub place_of_birth: OperatorBirthPlace,
    #[serde(alias = "DateOfBirth")]
    pub date_of_birth: String,
    #[serde(alias = "Race")]
    pub race: OperatorRace,
    #[serde(alias = "Height")]
    pub height: String,
    #[serde(alias = "InfectionStatus")]
    pub infection_status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalExam {
    #[serde(alias = "PhysicalStrength")]
    pub physical_strength: String,
    #[serde(alias = "Mobility")]
    pub mobility: String,
    #[serde(alias = "PhysicalResilience")]
    pub physical_resilience: String,
    #[serde(alias = "TacticalAcumen")]
    pub tactical_acumen: String,
    #[serde(alias = "CombatSkill")]
    pub combat_skill: String,
    #[serde(alias = "OriginiumArtsAssimilation")]
    pub originium_arts_assimilation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorProfile {
    #[serde(alias = "BasicInfo")]
    pub basic_info: BasicInfo,
    #[serde(alias = "PhysicalExam")]
    pub physical_exam: PhysicalExam,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookRewardItem {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "Count")]
    pub count: i32,
    #[serde(rename = "type", alias = "Type_")]
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMission {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "Sort")]
    pub sort: i32,
    #[serde(alias = "PowerId")]
    pub power_id: String,
    #[serde(alias = "PowerName")]
    pub power_name: String,
    #[serde(alias = "Item")]
    pub item: HandbookRewardItem,
    #[serde(alias = "FavorPoint")]
    pub favor_point: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookDisplayCondition {
    #[serde(alias = "CharId")]
    pub char_id: String,
    #[serde(alias = "ConditionCharId")]
    pub condition_char_id: String,
    #[serde(rename = "type", alias = "Type_")]
    pub condition_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStageTime {
    #[serde(alias = "Timestamp")]
    pub timestamp: i64,
    #[serde(alias = "CharSet")]
    pub char_set: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStory {
    #[serde(alias = "StoryText")]
    pub story_text: String,
    #[serde(alias = "UnLockType")]
    pub unlock_type: String,
    #[serde(alias = "UnLockParam")]
    pub un_lock_param: String,
    #[serde(alias = "UnLockString")]
    pub un_lock_string: String,
    #[serde(alias = "PatchIdList")]
    pub patch_id_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStoryTextAudio {
    #[serde(alias = "Stories")]
    pub stories: Vec<HandbookStory>,
    #[serde(alias = "StoryTitle")]
    pub story_title: String,
    #[serde(alias = "UnLockorNot")]
    pub un_lockor_not: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookUnlockParam {
    #[serde(alias = "UnlockType")]
    pub unlock_type: String,
    #[serde(alias = "UnlockParam1")]
    pub unlock_param1: Option<String>,
    #[serde(alias = "UnlockParam2")]
    pub unlock_param2: Option<String>,
    #[serde(alias = "UnlockParam3")]
    pub unlock_param3: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookAvgEntry {
    #[serde(alias = "StoryId")]
    pub story_id: String,
    #[serde(alias = "StorySetId")]
    pub story_set_id: String,
    #[serde(alias = "StorySort")]
    pub story_sort: i32,
    #[serde(alias = "StoryCanShow")]
    pub story_can_show: bool,
    #[serde(alias = "StoryIntro")]
    pub story_intro: String,
    #[serde(alias = "StoryInfo")]
    pub story_info: String,
    #[serde(alias = "StoryTxt")]
    pub story_txt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookAvgList {
    #[serde(alias = "StorySetId")]
    pub story_set_id: String,
    #[serde(alias = "StorySetName")]
    pub story_set_name: String,
    #[serde(alias = "SortId")]
    pub sort_id: i32,
    #[serde(alias = "StoryGetTime")]
    pub story_get_time: i64,
    #[serde(alias = "RewardItem")]
    pub reward_item: Vec<HandbookRewardItem>,
    #[serde(alias = "UnlockParam")]
    pub unlock_param: Vec<HandbookUnlockParam>,
    #[serde(alias = "AvgList")]
    pub avg_list: Vec<HandbookAvgEntry>,
    #[serde(alias = "CharId")]
    pub char_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookItem {
    #[serde(rename = "charID", alias = "CharID")]
    pub char_id: String,
    #[serde(alias = "InfoName")]
    pub info_name: String,
    #[serde(alias = "IsLimited")]
    pub is_limited: bool,
    #[serde(alias = "StoryTextAudio")]
    pub story_text_audio: Vec<HandbookStoryTextAudio>,
    #[serde(alias = "HandbookAvgList", default)]
    pub handbook_avg_list: Vec<HandbookAvgList>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NPCUnlockInfo {
    #[serde(alias = "UnLockType")]
    pub un_lock_type: String,
    #[serde(alias = "UnLockParam")]
    pub un_lock_param: String,
    #[serde(alias = "UnLockString")]
    pub un_lock_string: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookNPCItem {
    #[serde(alias = "NpcId")]
    pub npc_id: String,
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "Appellation")]
    pub appellation: String,
    #[serde(alias = "Profession")]
    pub profession: String,
    #[serde(alias = "IllustList")]
    pub illust_list: Option<Vec<String>>,
    #[serde(alias = "DesignerList")]
    pub designer_list: Option<Vec<String>>,
    #[serde(alias = "Cv")]
    pub cv: String,
    #[serde(alias = "DisplayNumber")]
    pub display_number: String,
    #[serde(alias = "NationId")]
    pub nation_id: Option<String>,
    #[serde(alias = "GroupId")]
    pub group_id: Option<String>,
    #[serde(alias = "TeamId")]
    pub team_id: Option<String>,
    #[serde(alias = "ResType")]
    pub res_type: String,
    #[serde(alias = "NpcShowAudioInfoFlag")]
    pub npc_show_audio_info_flag: bool,
    #[serde(
        alias = "UnlockDict",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub unlock_dict: HashMap<String, NPCUnlockInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStageData {
    #[serde(alias = "CharId", default)]
    pub char_id: String,
    #[serde(alias = "StageId", default)]
    pub stage_id: String,
    #[serde(alias = "LevelId", default)]
    pub level_id: String,
    #[serde(alias = "ZoneId", default)]
    pub zone_id: String,
    #[serde(alias = "Code", default)]
    pub code: String,
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "LoadingPicId", default)]
    pub loading_pic_id: String,
    #[serde(alias = "Description", default)]
    pub description: String,
    #[serde(alias = "UnlockParam", default)]
    pub unlock_param: Vec<HandbookUnlockParam>,
    #[serde(alias = "RewardItem", default)]
    pub reward_item: Vec<HandbookRewardItem>,
    #[serde(alias = "StageNameForShow", default)]
    pub stage_name_for_show: String,
    #[serde(alias = "ZoneNameForShow", default)]
    pub zone_name_for_show: String,
    #[serde(alias = "PicId", default)]
    pub pic_id: String,
    #[serde(alias = "StageGetTime", default)]
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

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HandbookTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub handbook_dict: HashMap<String, HandbookItem>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub npc_dict: HashMap<String, HandbookNPCItem>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub team_mission_list: HashMap<String, TeamMission>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub handbook_display_condition_list: HashMap<String, HandbookDisplayCondition>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub handbook_stage_data: HashMap<String, HandbookStageData>,
    #[serde(default)]
    pub handbook_stage_time: Vec<HandbookStageTime>,
}
