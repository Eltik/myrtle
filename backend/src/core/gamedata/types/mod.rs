use std::collections::{HashMap, HashSet};

pub mod activity;
pub mod audio;
pub mod building;
pub mod campaign;
pub mod chapter;
pub mod chibi;
pub mod climb_tower;
pub mod consts;
pub mod enemy;
pub mod enemy_stages;
pub mod event_shop;
pub mod gacha;
pub mod gacha_detail;
pub mod handbook;
pub mod level;
pub mod material;
pub mod medal;
pub mod mission;
pub mod mission_archive;
pub mod module;
pub mod operator;
pub mod range;
pub mod retro;
pub mod roguelike;
pub mod sandbox_universe;
pub mod serde_helpers;
pub mod shop;
pub mod skill;
pub mod skin;
pub mod stage;
pub mod stage_evidence;
pub mod stage_index;
pub mod stage_universe;
pub mod story_review;
pub mod story_review_meta;
pub mod trust;
pub mod voice;
pub mod zone;

use activity::ActivityBasicInfo;
use building::BuildingDataFile;
use campaign::CampaignRotations;
use chibi::ChibiData;
use consts::GameDataConst;
use enemy::EnemyHandbook;
use enemy_stages::EnemyStageIndex;
use gacha::GachaData;
use handbook::Handbook;
use material::Materials;
use medal::MedalData;
use module::Modules;
use operator::Operator;
use range::Ranges;
use retro::RetroAct;
use roguelike::RoguelikeGameData;
use sandbox_universe::SandboxUniverse;
use shop::{SkinListing, SkinWindow};
use skill::Skill;
use skin::SkinData;
use stage::Stage;
use stage_index::StageIndex;
use stage_universe::StageUniverse;
use story_review::StoryReviewGroup;
use story_review_meta::StoryReviewMetaTableFile;
use trust::Favor;
use voice::Voices;
use zone::Zone;

#[derive(Debug, Clone, Default)]
pub struct GameData {
    pub operators: HashMap<String, Operator>,
    pub skills: HashMap<String, Skill>,
    pub materials: Materials,
    pub modules: Modules,
    pub skins: SkinData,
    pub handbook: Handbook,
    pub ranges: Ranges,
    pub favor: Favor,
    pub voices: Voices,
    pub gacha: GachaData,
    pub chibis: ChibiData,
    pub enemy_chibis: ChibiData,
    pub zones: HashMap<String, Zone>,
    pub stages: HashMap<String, Stage>,
    pub activities: HashMap<String, ActivityBasicInfo>,
    /// Skin ids referenced by `activity_table` (event-reward skins), see
    /// [`activity::scan_skin_refs`].
    pub activity_skin_refs: HashSet<String>,
    /// Activity id -> its Originite Prime stages, see [`activity::op_stages_by_activity`].
    pub activity_op_stages: HashMap<String, Vec<activity::OpStage>>,
    /// Activity id -> its farming stages and their drops, see [`activity::farm_stages_by_activity`].
    pub activity_farm_stages: HashMap<String, Vec<activity::FarmStage>>,
    /// Activity id -> event currency its missions pay, see [`activity::mission_tokens_by_activity`].
    pub activity_mission_tokens: HashMap<String, i32>,
    /// Activity id -> its token shop as the game server serves it, see [`event_shop`].
    pub event_shops: HashMap<String, event_shop::EventShopData>,
    pub retro_acts: HashMap<String, RetroAct>,
    /// The Archives library index (`story_review_table`), keyed by group id.
    pub story_reviews: HashMap<String, StoryReviewGroup>,
    /// The event archives (`story_review_meta_table`), layout and content as
    /// the table carries them; the join to a story group is done once at
    /// story-index build.
    pub story_archives: StoryReviewMetaTableFile,
    /// The recordings shelves (`activity_table.MissionArchives`), keyed by
    /// topic id. EN carries one, `mission_archive_main_14`.
    pub mission_archives: HashMap<String, mission_archive::MissionArchive>,
    /// The soundtrack tables (`audio_data.Musics`/`BgmBanks`/`BankAlias`),
    /// which give a story group its theme and an archived track its clips.
    pub music: audio::MusicBanks,
    /// Main story chapters (`chapter_table`), keyed by chapter id.
    pub chapters: HashMap<String, chapter::Chapter>,
    /// Main zone id -> chapter id (`zone_table.MainlineAdditionInfo`).
    pub zone_chapters: HashMap<String, String>,
    /// Main zone id -> `ZoneOpenTime`, only the zones that carry a real one
    /// (5 of 17 on EN: `main_10` through `main_14`).
    pub zone_open_times: HashMap<String, i64>,
    /// `stage_table.Storylines`: the game's themed shelves over the Archives,
    /// in table order (14 on EN, `mainLine` plus 13 `ssLine_*`).
    pub storylines: Vec<stage::Storyline>,
    /// `stage_table.StorylineStorySets` keyed by `StorySetId`: the join from a
    /// storyline location to a `story_review_table` group (81 on EN).
    pub storyline_story_sets: HashMap<String, stage::StorylineStorySet>,
    /// Skin shop carousel windows (the promo layer), see [`shop`].
    pub skin_windows: Vec<SkinWindow>,
    /// Skin store listings from the recommend panel (the rerun record).
    pub skin_listings: Vec<SkinListing>,
    pub medals: MedalData,
    pub roguelike: RoguelikeGameData,
    pub enemies: EnemyHandbook,
    pub enemy_stage_index: EnemyStageIndex,
    pub stage_index: StageIndex,
    pub mode_levels: HashMap<String, String>,
    pub building: BuildingDataFile,
    pub stage_universe: StageUniverse,
    /// What a player's surviving mission, medal and unlock records prove about
    /// stages the client no longer keeps battle records for, see [`stage_evidence`].
    pub stage_evidence: stage_evidence::StageEvidenceIndex,
    pub sandbox_universe: SandboxUniverse,
    pub campaign_rotations: CampaignRotations,
    pub consts: GameDataConst,
    /// Daily and weekly mission chests (the LMD an account earns from dailies).
    pub missions: mission::MissionData,
    /// Tables that failed to deserialize and fell back to `T::default()`.
    /// One entry per table, `"<table>: <error>"`. Empty on a clean load.
    ///
    /// These are the quiet failures: `load_table_or_warn` swallows the error and
    /// hands back an empty table, so the site serves plausible-looking wrong data
    /// with only a `tracing::warn!` to show for it. Retained here so `/health` can
    /// report them and a reload can alert on them.
    pub table_warnings: Vec<String>,
}

impl GameData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_loaded(&self) -> bool {
        !self.operators.is_empty() && !self.skills.is_empty()
    }
}
