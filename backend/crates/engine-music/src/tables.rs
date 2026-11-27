use std::collections::{BTreeMap, HashMap};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

pub const TABLES: [&str; 9] = [
    "activity_table",
    "building_data",
    "crisis_table",
    "crisis_v2_table",
    "display_meta_table",
    "roguelike_topic_table",
    "sandbox_perm_table",
    "stage_table",
    "zone_table",
];

#[derive(Default)]
pub struct MusicTables {
    pub(crate) activity: ActivityTable,
    pub(crate) building: BuildingMusic,
    pub(crate) crisis: CrisisTable,
    pub(crate) crisis_v2: CrisisV2Table,
    pub(crate) display: DisplayMeta,
    pub(crate) rogue: RoguelikeTopics,
    pub(crate) sandbox: SandboxPerm,
    pub(crate) story_sets: StorySetTable,
    pub(crate) zones: ZoneMusicTable,
}

impl MusicTables {
    pub fn parse(read: impl FnMut(&str) -> Option<Vec<u8>>) -> (Self, Vec<String>) {
        let mut r = Reader {
            read,
            warnings: Vec::new(),
        };
        let tables = Self {
            activity: r.table("activity_table"),
            building: r.table("building_data"),
            crisis: r.table("crisis_table"),
            crisis_v2: r.table("crisis_v2_table"),
            display: r.table("display_meta_table"),
            rogue: r.table("roguelike_topic_table"),
            sandbox: r.table("sandbox_perm_table"),
            story_sets: r.table("stage_table"),
            zones: r.table("zone_table"),
        };
        (tables, r.warnings)
    }
}

struct Reader<F> {
    read: F,
    warnings: Vec<String>,
}

impl<F: FnMut(&str) -> Option<Vec<u8>>> Reader<F> {
    fn table<T: DeserializeOwned + Default>(&mut self, name: &str) -> T {
        let Some(bytes) = (self.read)(name) else {
            return T::default();
        };
        serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            self.warnings.push(format!("{name}: {e}"));
            T::default()
        })
    }
}

fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(T::deserialize(v).unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum KeyedOrMap<T> {
    Keyed(Vec<Keyed<T>>),
    Map(BTreeMap<String, T>),
}

impl<T> Default for KeyedOrMap<T> {
    fn default() -> Self {
        Self::Keyed(Vec::new())
    }
}

impl<T> KeyedOrMap<T> {
    pub(crate) fn values(&self) -> Vec<&T> {
        match self {
            Self::Keyed(v) => v.iter().map(|k| &k.value).collect(),
            Self::Map(m) => m.values().collect(),
        }
    }

    pub(crate) fn pairs(&self) -> Vec<(String, &T)> {
        match self {
            Self::Keyed(v) => v.iter().map(|k| (k.key.clone(), &k.value)).collect(),
            Self::Map(m) => m.iter().map(|(k, v)| (k.clone(), v)).collect(),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Keyed<T> {
    #[serde(default)]
    pub(crate) key: String,
    pub(crate) value: T,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct CrisisTable {
    #[serde(default)]
    pub(crate) season_info: Vec<CrisisSeason>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct CrisisSeason {
    #[serde(default)]
    pub(crate) season_id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) perm_bgm: Option<String>,
    #[serde(default)]
    pub(crate) perm_bgm_hard: Option<String>,
    #[serde(default)]
    pub(crate) start_ts: i64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct CrisisV2Table {
    #[serde(default)]
    pub(crate) season_info_data_map: KeyedOrMap<CrisisV2Season>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct CrisisV2Season {
    #[serde(default)]
    pub(crate) season_id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) season_bgm: Option<String>,
    #[serde(default)]
    pub(crate) season_bgm_challenge: Option<String>,
    #[serde(default)]
    pub(crate) start_ts: i64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ActivityTable {
    #[serde(default)]
    pub(crate) activity: BTreeMap<String, Vec<Keyed<ActivityMusic>>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ActivityMusic {
    #[serde(default, deserialize_with = "lenient")]
    pub(crate) const_data: Option<ActivityMusicConst>,
    #[serde(default, deserialize_with = "lenient")]
    pub(crate) music_data_map: Vec<ActivityMusicRow>,
    #[serde(default, deserialize_with = "lenient")]
    pub(crate) product_group_data_map: Vec<Keyed<ActivityProductGroup>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ActivityMusicConst {
    #[serde(default)]
    pub(crate) entry_music_id: Option<String>,
    #[serde(default)]
    pub(crate) boss_enter_bgm_key: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ActivityMusicRow {
    #[serde(default)]
    pub(crate) music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ActivityProductGroup {
    #[serde(default)]
    pub(crate) default_bgm_signal: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DisplayMeta {
    #[serde(default)]
    pub(crate) home_background_data: HomeBackgroundData,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct HomeBackgroundData {
    #[serde(default)]
    pub(crate) home_bg_data_list: Vec<HomeBg>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct HomeBg {
    #[serde(default)]
    pub(crate) bg_name: String,
    #[serde(default)]
    pub(crate) multi_form_list: Vec<HomeBgForm>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct HomeBgForm {
    #[serde(default)]
    pub(crate) bg_music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct BuildingMusic {
    #[serde(default)]
    pub(crate) music_data: BuildingMusicData,
    #[serde(default)]
    pub(crate) custom_data: BuildingCustom,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct BuildingMusicData {
    #[serde(default)]
    pub(crate) music_datas: KeyedOrMap<JukeboxEntry>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct JukeboxEntry {
    #[serde(default)]
    pub(crate) bgm_name: String,
    #[serde(default)]
    pub(crate) bgm_sort_id: i64,
    #[serde(default)]
    pub(crate) game_music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct BuildingCustom {
    #[serde(default)]
    pub(crate) furnitures: KeyedOrMap<FurnitureMusic>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct FurnitureMusic {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) music_id: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ZoneMusicTable {
    #[serde(default)]
    pub(crate) mainline_addition_info: KeyedOrMap<MainlineAddition>,
    #[serde(default)]
    pub(crate) mainline_zone_id_list: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct MainlineAddition {
    #[serde(default)]
    pub(crate) zone_id: String,
    #[serde(default)]
    pub(crate) game_music_id: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct StorySetTable {
    #[serde(default)]
    pub(crate) storyline_story_sets: KeyedOrMap<StorySetMusic>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct StorySetMusic {
    #[serde(default)]
    pub(crate) game_music_id: String,
    #[serde(default)]
    pub(crate) relevant_activity_id: Option<String>,
    #[serde(default)]
    pub(crate) mainline_data: Option<StorySetZone>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct StorySetZone {
    #[serde(default)]
    pub(crate) zone_id: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RoguelikeTopics {
    #[serde(default)]
    pub(crate) details: KeyedOrMap<RoguelikeDetail>,
    #[serde(default)]
    pub(crate) topics: KeyedOrMap<RoguelikeTopic>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RoguelikeTopic {
    #[serde(default)]
    pub(crate) start_time: i64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RoguelikeDetail {
    #[serde(default)]
    pub(crate) zones: KeyedOrMap<RoguelikeZone>,
    #[serde(default)]
    pub(crate) game_const: RoguelikeConst,
    #[serde(default)]
    pub(crate) variation_data: KeyedOrMap<RoguelikeVariation>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RoguelikeZone {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) bgm_signal: Option<String>,
    #[serde(default)]
    pub(crate) bgm_signal_with_low_san: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RoguelikeConst {
    #[serde(default)]
    pub(crate) sp_zone_shop_bgm_signal: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct RoguelikeVariation {
    #[serde(default)]
    pub(crate) outer_name: String,
    #[serde(default)]
    pub(crate) sound: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SandboxPerm {
    #[serde(default)]
    pub(crate) basic_info: KeyedOrMap<SandboxTopic>,
    #[serde(default)]
    pub(crate) detail: SandboxDetail,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SandboxTopic {
    #[serde(default)]
    pub(crate) topic_id: String,
    #[serde(default)]
    pub(crate) topic_name: String,
}

#[derive(Deserialize, Default)]
pub(crate) struct SandboxDetail {
    #[serde(rename = "SANDBOX_V2", default)]
    pub(crate) sandbox_v2: Vec<SandboxKeyed>,
    #[serde(rename = "SANDBOX_V3", default)]
    pub(crate) sandbox_v3: Vec<SandboxKeyed>,
}

#[derive(Deserialize, Default)]
pub(crate) struct SandboxKeyed {
    #[serde(default)]
    pub(crate) key: String,
    #[serde(default)]
    pub(crate) value: SandboxRun,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SandboxRun {
    #[serde(default)]
    pub(crate) basic_const: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) game_const: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) archive_music_unlock_data: KeyedOrMap<SandboxMusic>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SandboxMusic {
    #[serde(default)]
    pub(crate) music_id: String,
}
