use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceType {
    HomePlace,
    NewYear,
    Greeting,
    Anniversary,
    HomeShow,
    HomeWait,
    Gacha,
    LevelUp,
    Squad,
    SquadFirst,
    BattleStart,
    BattleFaceEnemy,
    BattleSelect,
    BattlePlace,
    BattleSkill1,
    BattleSkill2,
    BattleSkill3,
    BattleSkill4,
    FourStar,
    ThreeStar,
    TwoStar,
    Lose,
    BuildingPlace,
    BuildingTouching,
    BuildingFavorBubble,
    LoadingPanel,
}

impl Default for PlaceType {
    fn default() -> Self {
        Self::HomePlace
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LangType {
    CnMandarin,
    Jp,
    Kr,
    En,
    Rus,
    Ita,
    CnTopolect,
    Linkage,
    Ger,
}

impl Default for LangType {
    fn default() -> Self {
        Self::CnMandarin
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VoiceType {
    #[serde(rename = "ENUM")]
    Enum,
    #[serde(rename = "ONLY_TEXT")]
    OnlyText,
}

impl Default for VoiceType {
    fn default() -> Self {
        Self::Enum
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum UnlockType {
    Direct,
    Favor,
    Awake,
}

impl Default for UnlockType {
    fn default() -> Self {
        Self::Direct
    }
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockParam {
    pub value_str: Option<String>,
    pub value_int: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceData {
    pub voice_url: Option<String>,
    pub language: Option<LangType>,
    pub cv_name: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharExtraWord {
    pub word_key: String,
    pub char_id: String,
    pub voice_id: String,
    pub voice_text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLangTypeInfo {
    pub name: String,
    pub group_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLangGroupType {
    pub name: String,
    pub members: Vec<LangType>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTimeWithType {
    pub timestamp: i64,
    pub char_set: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FesTimeInterval {
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FesTimeData {
    pub time_type: String,
    pub interval: FesTimeInterval,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FesVoiceData {
    pub show_type: PlaceType,
    pub time_data: Vec<FesTimeData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FesVoiceWeight {
    pub show_type: PlaceType,
    pub weight: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraVoiceConfigData {
    pub voice_id: String,
    pub valid_voice_lang: Vec<LangType>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLangDictEntry {
    pub wordkey: String,
    pub voice_lang_type: LangType,
    pub cv_name: Vec<String>,
    pub voice_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLang {
    pub wordkeys: Vec<String>,
    pub char_id: String,
    pub dict: HashMap<String, VoiceLangDictEntry>,
}

// ============================================================================
// RawVoice
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawVoice {
    pub char_word_id: String,
    pub word_key: String,
    pub char_id: String,
    pub voice_id: String,
    pub voice_text: String,
    pub voice_title: String,
    pub voice_index: i32,
    pub voice_type: VoiceType,
    pub unlock_type: UnlockType,
    pub unlock_param: Vec<UnlockParam>,
    pub lock_description: Option<String>,
    pub place_type: PlaceType,
    pub voice_asset: String,
}

// ============================================================================
// Voice
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Voice {
    pub char_word_id: String,
    pub word_key: String,
    pub char_id: String,
    pub voice_id: String,
    pub voice_text: String,
    pub voice_title: String,
    pub voice_index: i32,
    pub voice_type: VoiceType,
    pub unlock_type: UnlockType,
    pub unlock_param: Vec<UnlockParam>,
    pub lock_description: Option<String>,
    pub place_type: PlaceType,
    pub voice_asset: String,
    // Added fields
    pub id: Option<String>,
    pub data: Option<Vec<VoiceData>>,
    pub languages: Option<Vec<LangType>>,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Voices {
    pub char_words: HashMap<String, Voice>,
    pub char_extra_words: HashMap<String, CharExtraWord>,
    pub voice_lang_dict: HashMap<String, VoiceLang>,
    pub default_lang_type: String,
    pub new_tag_list: Vec<String>,
    pub voice_lang_type_dict: HashMap<LangType, VoiceLangTypeInfo>,
    pub voice_lang_group_type_dict: HashMap<String, VoiceLangGroupType>,
    pub char_default_type_dict: HashMap<String, LangType>,
    pub start_time_with_type_dict: HashMap<LangType, Vec<StartTimeWithType>>,
    pub display_group_type_list: Vec<String>,
    pub display_type_list: Vec<LangType>,
    pub play_voice_range: PlaceType,
    pub fes_voice_data: HashMap<PlaceType, FesVoiceData>,
    pub fes_voice_weight: HashMap<PlaceType, FesVoiceWeight>,
    pub extra_voice_config_data: HashMap<String, ExtraVoiceConfigData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawVoices {
    pub char_words: HashMap<String, RawVoice>,
    pub char_extra_words: HashMap<String, CharExtraWord>,
    pub voice_lang_dict: HashMap<String, VoiceLang>,
    pub default_lang_type: String,
    pub new_tag_list: Vec<String>,
    pub voice_lang_type_dict: HashMap<LangType, VoiceLangTypeInfo>,
    pub voice_lang_group_type_dict: HashMap<String, VoiceLangGroupType>,
    pub char_default_type_dict: HashMap<String, LangType>,
    pub start_time_with_type_dict: HashMap<LangType, Vec<StartTimeWithType>>,
    pub display_group_type_list: Vec<String>,
    pub display_type_list: Vec<LangType>,
    pub play_voice_range: PlaceType,
    pub fes_voice_data: HashMap<PlaceType, FesVoiceData>,
    pub fes_voice_weight: HashMap<PlaceType, FesVoiceWeight>,
    pub extra_voice_config_data: HashMap<String, ExtraVoiceConfigData>,
}
