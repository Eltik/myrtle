use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from account/syncData endpoint
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserResponse {
    pub result: i32,
    pub ts: i64,
    pub user: Option<User>,
}

/// Main user data structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct User {
    pub dungeon: Dungeon,
    pub activity: serde_json::Value, // Complex, varies by event
    pub status: UserStatus,
    pub troop: Troop,
    pub npc_audio: HashMap<String, NpcAudioInfo>,
    pub recruit: Recruit,
    pub push_flags: PushFlags,
    pub equipment: Equipment,
    pub skin: UserSkin,
    pub shop: Shop,
    pub mission: Mission,
    pub social: Social,
    pub building: Building,
    pub dex_nav: DexNav,
    pub crisis: Crisis,
    pub tshop: HashMap<String, TShopEntry>,
    pub gacha: Gacha,
    pub backflow: Backflow,
    pub mainline: Mainline,
    pub avatar: HashMap<String, AvatarEntry>,
    pub background: Background,
    pub home_theme: HomeTheme,
    pub rlv2: Rlv2,
    pub deep_sea: DeepSea,
    pub tower: Tower,
    pub siracusa_map: SiracusaMap,
    pub storyreview: StoryReview,
    pub medal: Medal,
    pub april_fool: HashMap<String, AprilFoolEntry>,
    pub retro: Retro,
    pub charm: Charm,
    pub carousel: Carousel,
    pub consumable: HashMap<String, HashMap<String, ConsumableEntry>>,
    pub event: Event,
    pub collection_reward: CollectionReward,
    pub check_in: CheckIn,
    pub car: Car,
    pub open_server: OpenServer,
    pub campaigns_v2: CampaignsV2,
    pub inventory: HashMap<String, serde_json::Value>,
    pub limited_buff: LimitedBuff,
    pub ticket: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Dungeon {
    pub stages: HashMap<String, StageData>,
    pub cow_level: HashMap<String, CowLevelEntry>,
    #[serde(default)]
    pub mainline_banned_stages: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StageData {
    pub stage_id: String,
    pub complete_times: i32,
    pub start_times: i32,
    pub practice_times: i32,
    pub state: i32,
    pub has_battle_replay: i32,
    pub no_cost_cnt: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CowLevelEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub val: Vec<serde_json::Value>,
    pub fts: i64,
    pub rts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserStatus {
    pub nick_name: String,
    pub nick_number: String,
    pub level: i32,
    pub exp: i32,
    pub social_point: i32,
    pub gacha_ticket: i32,
    pub classic_gacha_ticket: i32,
    pub ten_gacha_ticket: i32,
    pub classic_ten_gacha_ticket: i32,
    pub instant_finish_ticket: i32,
    pub hgg_shard: i32,
    pub lgg_shard: i32,
    pub classic_shard: i32,
    pub recruit_license: i32,
    pub progress: i32,
    pub buy_ap_remain_times: i32,
    pub ap_limit_up_flag: i32,
    pub uid: String,
    pub flags: HashMap<String, i32>,
    pub ap: i32,
    pub max_ap: i32,
    pub pay_diamond: i32,
    pub free_diamond: i32,
    pub diamond_shard: i32,
    pub gold: i64,
    pub practice_ticket: i32,
    pub last_refresh_ts: i64,
    pub last_ap_add_time: i64,
    pub main_stage_progress: String,
    pub register_ts: i64,
    pub last_online_ts: i64,
    pub server_name: String,
    pub avatar_id: String,
    pub avatar: Avatar,
    pub resume: String,
    pub friend_num_limit: i32,
    pub monthly_subscription_start_time: i64,
    pub monthly_subscription_end_time: i64,
    pub tip_monthly_card_expire_ts: i64,
    pub secretary: String,
    pub secretary_skin_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Avatar {
    #[serde(rename = "type")]
    pub avatar_type: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Troop {
    pub cur_char_inst_id: i32,
    pub cur_squad_count: i32,
    pub squads: HashMap<String, Squad>,
    pub chars: HashMap<String, CharacterData>,
    pub char_group: HashMap<String, CharGroup>,
    pub char_mission: HashMap<String, HashMap<String, i32>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Squad {
    pub squad_id: String,
    pub name: String,
    pub slots: Vec<Option<SquadSlot>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SquadSlot {
    pub char_inst_id: i32,
    pub skill_index: i32,
    pub current_equip: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CharacterData {
    pub inst_id: i32,
    pub char_id: String,
    pub favor_point: i32,
    pub potential_rank: i32,
    pub main_skill_lvl: i32,
    pub skin: Option<String>,
    pub level: i32,
    pub exp: i32,
    pub evolve_phase: i32,
    pub default_skill_index: i32,
    pub gain_time: i64,
    pub current_tmpl: Option<String>,
    pub tmpl: Option<HashMap<String, CharacterTemplate>>,
    pub skills: Vec<CharacterSkill>,
    pub voice_lan: String,
    pub current_equip: Option<String>,
    pub equip: HashMap<String, EquipData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#static: Option<serde_json::Value>, // Added by formatUser
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CharacterTemplate {
    pub equip: HashMap<String, EquipData>,
    pub skills: Vec<CharacterSkill>,
    pub skin_id: String,
    pub current_equip: String,
    pub default_skill_index: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CharacterSkill {
    pub skill_id: String,
    pub unlock: i32,
    pub state: i32,
    pub specialize_level: i32,
    pub complete_upgrade_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#static: Option<serde_json::Value>, // Added by formatUser
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EquipData {
    pub hide: i32,
    pub locked: i32,
    pub level: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CharGroup {
    pub favor_point: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NpcAudioInfo {
    pub npc_show_audio_info_flag: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Recruit {
    pub normal: RecruitNormal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecruitNormal {
    pub slots: HashMap<String, RecruitSlot>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecruitSlot {
    pub state: i32,
    pub tags: Vec<i32>,
    pub select_tags: Vec<SelectTag>,
    pub start_ts: i64,
    pub duration_in_sec: i64,
    pub max_finish_ts: i64,
    pub real_finish_ts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SelectTag {
    pub tag_id: i32,
    pub pick: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PushFlags {
    pub has_gifts: i32,
    pub has_friend_request: i32,
    pub has_clues: i32,
    pub has_free_level_gp: i32,
    pub status: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Equipment {
    pub missions: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserSkin {
    pub character_skins: HashMap<String, i32>,
    pub skin_ts: HashMap<String, i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Shop {
    #[serde(rename = "LS")]
    pub ls: ShopLS,
    #[serde(rename = "HS")]
    pub hs: ShopHS,
    #[serde(rename = "ES")]
    pub es: ShopES,
    #[serde(rename = "CASH")]
    pub cash: ShopCash,
    #[serde(rename = "GP")]
    pub gp: ShopGP,
    #[serde(rename = "FURNI")]
    pub furni: ShopFurni,
    #[serde(rename = "SOCIAL")]
    pub social: ShopSocial,
    #[serde(rename = "CLASSIC")]
    pub classic: ShopClassic,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopLS {
    pub cur_shop_id: String,
    pub cur_group_id: String,
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopHS {
    pub cur_shop_id: String,
    pub info: Vec<ShopInfo>,
    pub progress_info: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopES {
    pub cur_shop_id: String,
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopCash {
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopGP {
    pub one_time: ShopInfoContainer,
    pub level: ShopInfoContainer,
    pub weekly: ShopGroupInfoContainer,
    pub monthly: ShopGroupInfoContainer,
    pub choose: ShopInfoContainer,
    pub backflow: ShopInfoContainer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopInfoContainer {
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopGroupInfoContainer {
    pub cur_group_id: String,
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopFurni {
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopSocial {
    pub cur_shop_id: String,
    pub info: Vec<ShopInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopClassic {
    pub info: Vec<ShopInfo>,
    pub progress_info: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShopInfo {
    pub id: String,
    pub count: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Mission {
    pub missions: MissionCategories,
    pub mission_rewards: MissionRewards,
    pub mission_groups: HashMap<String, i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", default)]
pub struct MissionCategories {
    pub openserver: HashMap<String, MissionData>,
    pub daily: HashMap<String, MissionData>,
    pub weekly: HashMap<String, MissionData>,
    pub guide: HashMap<String, MissionData>,
    pub main: HashMap<String, MissionData>,
    pub activity: HashMap<String, MissionData>,
    pub sub: HashMap<String, MissionData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MissionData {
    pub state: i32,
    pub progress: Vec<MissionProgress>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MissionProgress {
    pub target: i32,
    pub value: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MissionRewards {
    pub daily_point: i32,
    pub weekly_point: i32,
    pub rewards: MissionRewardCategories,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", default)]
pub struct MissionRewardCategories {
    pub daily: HashMap<String, i32>,
    pub weekly: HashMap<String, i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Social {
    pub assist_char_list: Vec<Option<AssistChar>>,
    pub yesterday_reward: YesterdayReward,
    pub y_crisis_ss: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AssistChar {
    pub char_inst_id: i32,
    pub skill_index: i32,
    pub current_equip: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct YesterdayReward {
    pub can_receive: i32,
    pub assist_amount: i32,
    pub comfort_amount: i32,
    pub first: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Building {
    pub status: BuildingStatus,
    pub chars: HashMap<String, BuildingChar>,
    pub room_slots: HashMap<String, RoomSlot>,
    pub rooms: BuildingRooms,
    pub furniture: HashMap<String, FurnitureEntry>,
    pub assist: Vec<i32>,
    pub diy_present_solutions: serde_json::Value,
    pub solution: BuildingSolution,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildingStatus {
    pub labor: Labor,
    pub workshop: WorkshopStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Labor {
    pub buff_speed: f64,
    pub process_point: f64,
    pub value: i32,
    pub last_update_time: i64,
    pub max_value: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkshopStatus {
    pub bonus_active: i32,
    pub bonus: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildingChar {
    pub char_id: String,
    pub last_ap_add_time: i64,
    pub ap: i32,
    pub room_slot_id: String,
    pub index: i32,
    pub change_scale: i32,
    pub bubble: Bubble,
    pub work_time: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bubble {
    pub normal: BubbleData,
    pub assist: BubbleData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BubbleData {
    pub add: i32,
    pub ts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RoomSlot {
    pub level: i32,
    pub state: i32,
    pub room_id: String,
    pub char_inst_ids: Vec<i32>,
    pub complete_construct_time: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", default)]
pub struct BuildingRooms {
    pub control: HashMap<String, serde_json::Value>,
    pub elevator: HashMap<String, serde_json::Value>,
    pub power: HashMap<String, serde_json::Value>,
    pub manufacture: HashMap<String, serde_json::Value>,
    pub trading: HashMap<String, serde_json::Value>,
    pub dormitory: HashMap<String, serde_json::Value>,
    pub corridor: HashMap<String, serde_json::Value>,
    pub workshop: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FurnitureEntry {
    pub count: i32,
    pub in_use: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildingSolution {
    pub furniture_ts: HashMap<String, i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DexNav {
    pub character: HashMap<String, DexNavCharacter>,
    pub formula: DexNavFormula,
    pub team_v2: HashMap<String, HashMap<String, i32>>,
    pub enemy: DexNavEnemy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DexNavCharacter {
    pub char_inst_id: i32,
    pub count: i32,
    pub classic_count: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DexNavFormula {
    pub shop: HashMap<String, i32>,
    pub manufacture: HashMap<String, i32>,
    pub workshop: HashMap<String, i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DexNavEnemy {
    pub enemies: HashMap<String, i32>,
    pub stage: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Crisis {
    pub current: String,
    pub lst: i64,
    pub nst: i64,
    pub map: HashMap<String, CrisisMap>,
    pub shop: CrisisShop,
    pub training: CrisisTraining,
    pub season: serde_json::Value,
    #[serde(default)]
    pub r#box: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrisisMap {
    pub rank: i32,
    pub confirmed: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrisisShop {
    pub coin: i32,
    #[serde(default)]
    pub info: Vec<serde_json::Value>,
    pub progress_info: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrisisTraining {
    pub current_stage: Vec<String>,
    pub stage: HashMap<String, CrisisTrainingStage>,
    pub nst: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrisisTrainingStage {
    pub point: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TShopEntry {
    pub coin: i32,
    #[serde(default)]
    pub info: Vec<serde_json::Value>,
    pub progress_info: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Gacha {
    pub newbee: GachaNewbee,
    pub normal: HashMap<String, GachaNormal>,
    pub attain: serde_json::Value,
    pub limit: HashMap<String, GachaLimit>,
    pub single: HashMap<String, GachaSingle>,
    pub fes_classic: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GachaNewbee {
    pub open_flag: i32,
    pub cnt: i32,
    pub pool_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GachaNormal {
    pub cnt: i32,
    pub max_cnt: i32,
    pub rarity: i32,
    pub avail: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GachaLimit {
    pub least_free: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GachaSingle {
    pub single_ensure_cnt: i32,
    pub single_ensure_use: bool,
    pub single_ensure_char: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Backflow {
    pub open: bool,
    pub current: Option<serde_json::Value>,
    pub current_v2: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Mainline {
    pub record: serde_json::Value,
    #[serde(default)]
    pub cache: Vec<serde_json::Value>,
    pub version: i32,
    pub additional_mission: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AvatarEntry {
    pub ts: i64,
    pub src: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Background {
    pub selected: String,
    pub bgs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HomeTheme {
    pub selected: String,
    pub themes: HashMap<String, HomeThemeEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HomeThemeEntry {
    pub unlock: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rlv2 {
    pub current: serde_json::Value,
    pub outer: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeepSea {
    pub places: HashMap<String, i32>,
    pub nodes: HashMap<String, i32>,
    pub choices: HashMap<String, Vec<i32>>,
    pub events: serde_json::Value,
    pub treasures: serde_json::Value,
    pub stories: serde_json::Value,
    pub tech_trees: serde_json::Value,
    pub logs: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Tower {
    pub current: serde_json::Value,
    pub outer: serde_json::Value,
    pub season: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SiracusaMap {
    pub select: Option<serde_json::Value>,
    pub card: serde_json::Value,
    pub opera: SiracusaOpera,
    pub area: HashMap<String, i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SiracusaOpera {
    pub total: i32,
    pub show: Option<serde_json::Value>,
    pub release: HashMap<String, i32>,
    pub like: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StoryReview {
    pub groups: HashMap<String, StoryReviewGroup>,
    pub tags: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StoryReviewGroup {
    pub rts: i64,
    #[serde(default)]
    pub stories: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Medal {
    pub medals: HashMap<String, MedalEntry>,
    pub custom: MedalCustom,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MedalEntry {
    pub id: String,
    pub val: serde_json::Value, // Can be nested arrays like [[23,30]]
    pub fts: i64,
    pub rts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MedalCustom {
    pub current_index: Option<serde_json::Value>,
    pub customs: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AprilFoolEntry {
    pub stages: serde_json::Value,
    pub live_endings: Option<serde_json::Value>,
    pub camera_lv: Option<i32>,
    pub fans: Option<i32>,
    pub posts: Option<i32>,
    pub missions: Option<HashMap<String, AprilFoolMission>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AprilFoolMission {
    pub value: i32,
    pub target: i32,
    pub finished: bool,
    pub has_recv: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Retro {
    pub coin: i32,
    pub supplement: i32,
    pub block: HashMap<String, RetroBlock>,
    pub trail: HashMap<String, HashMap<String, i32>>,
    pub lst: i64,
    pub nst: i64,
    #[serde(default)]
    pub reward_perm: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RetroBlock {
    pub locked: i32,
    pub open: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Charm {
    pub charms: HashMap<String, i32>,
    #[serde(default)]
    pub squad: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Carousel {
    pub furniture_shop: CarouselFurnitureShop,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CarouselFurnitureShop {
    pub goods: serde_json::Value,
    pub groups: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConsumableEntry {
    pub ts: i64,
    pub count: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Event {
    pub building: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CollectionReward {
    pub team: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckIn {
    pub can_check_in: i32,
    pub check_in_group_id: String,
    pub check_in_reward_index: i32,
    pub check_in_history: Vec<i32>,
    pub newbie_package: NewbiePackage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NewbiePackage {
    pub open: bool,
    pub group_id: String,
    pub finish: i32,
    pub stop_sale: i32,
    pub check_in_history: Vec<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Car {
    pub battle_car: HashMap<String, Option<serde_json::Value>>,
    pub exhibition_car: HashMap<String, Option<serde_json::Value>>,
    pub accessories: HashMap<String, Option<serde_json::Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenServer {
    pub check_in: OpenServerCheckIn,
    pub chain_login: ChainLogin,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenServerCheckIn {
    pub is_available: bool,
    pub history: Vec<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChainLogin {
    pub is_available: bool,
    pub now_index: i32,
    pub history: Vec<i32>,
    pub final_reward: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CampaignsV2 {
    pub campaign_current_fee: i32,
    pub campaign_total_fee: i32,
    pub open: CampaignsOpen,
    pub instances: HashMap<String, CampaignInstance>,
    pub missions: HashMap<String, i32>,
    pub last_refresh_ts: i64,
    pub sweep_max_kills: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CampaignsOpen {
    pub permanent: Vec<String>,
    pub rotate: String,
    pub r_group: String,
    pub training: Vec<String>,
    pub t_group: String,
    pub t_all_open: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CampaignInstance {
    pub max_kills: i32,
    pub reward_status: Vec<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LimitedBuff {
    pub daily_usage: serde_json::Value,
    pub inventory: serde_json::Value,
}
