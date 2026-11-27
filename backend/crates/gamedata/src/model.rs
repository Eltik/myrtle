use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::{
    integrity::SkippedTable,
    tables::{
        activity::{ActivityBasicInfo, FarmStage, MissionArchive, OpStage},
        audio::{EnemyAudio, MusicBanks},
        autochess::AutoChessBond,
        building::BuildingDataFile,
        campaign::CampaignRotations,
        chapter::Chapter,
        consts::GameDataConst,
        display_meta::HomeBackground,
        enemy::EnemyHandbook,
        gacha::GachaData,
        handbook::Handbook,
        handbook_team::HandbookTeam,
        item::{RecommendTag, SkinListing, SkinWindow},
        material::Materials,
        medal::MedalData,
        mission::MissionData,
        module::Modules,
        operator::Operator,
        range::Ranges,
        roguelike::RoguelikeGameData,
        skill::Skill,
        stage::{RetroAct, Stage, Storyline, StorylineStorySet},
        story::{StoryReviewGroup, StoryReviewMetaTableFile},
        trust::Favor,
        voice::Voices,
        zone::Zone,
    },
};
use myrtle_types::Server;
use serde::{Deserialize, Serialize};

pub mod chibi;
pub mod enemy_stages;
pub mod event_shop;
pub mod sandbox_universe;
pub mod skin;
pub mod stage_evidence;
pub mod stage_index;
pub mod stage_universe;

use chibi::ChibiData;
use enemy_stages::EnemyStageIndex;
use event_shop::EventShopData;
use sandbox_universe::SandboxUniverse;
use skin::SkinData;
use stage_evidence::StageEvidenceIndex;
use stage_index::StageIndex;
use stage_universe::StageUniverse;

use crate::{assets::AssetIndex, index::squad::SquadIndex};

/// Bump when model changes
pub const MODEL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameVersion {
    pub client: String,
    pub res: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMeta {
    pub server: Server,
    pub game_version: GameVersion,
    pub schema_version: u32,
    pub skipped_tables: Vec<SkippedTable>,
    pub table_warnings: Vec<String>,
}

impl Default for ModelMeta {
    fn default() -> Self {
        Self {
            server: Server::EN,
            game_version: GameVersion::default(),
            schema_version: MODEL_SCHEMA_VERSION,
            skipped_tables: Vec::new(),
            table_warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GameModel {
    pub meta: ModelMeta,
    pub assets: AssetIndex,
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
    pub activity_skin_refs: HashSet<String>,
    pub activity_op_stages: HashMap<String, Vec<OpStage>>,
    pub activity_farm_stages: HashMap<String, Vec<FarmStage>>,
    pub activity_mission_tokens: HashMap<String, i32>,
    pub event_shops: HashMap<String, EventShopData>,
    pub retro_acts: HashMap<String, RetroAct>,
    pub story_reviews: HashMap<String, StoryReviewGroup>,
    pub story_archives: StoryReviewMetaTableFile,
    pub mission_archives: HashMap<String, MissionArchive>,
    pub music: MusicBanks,
    pub chapters: HashMap<String, Chapter>,
    pub zone_chapters: HashMap<String, String>,
    pub zone_open_times: HashMap<String, i64>,
    pub storylines: Vec<Storyline>,
    pub storyline_story_sets: HashMap<String, StorylineStorySet>,
    pub skin_windows: Vec<SkinWindow>,
    pub skin_listings: Vec<SkinListing>,
    pub recommend_tags: Vec<RecommendTag>,
    pub medals: MedalData,
    pub roguelike: RoguelikeGameData,
    pub enemies: EnemyHandbook,
    pub enemy_audio: HashMap<String, Vec<EnemyAudio>>,
    pub enemy_stage_index: EnemyStageIndex,
    pub stage_index: StageIndex,
    pub mode_levels: HashMap<String, String>,
    pub building: BuildingDataFile,
    pub stage_universe: StageUniverse,
    pub stage_evidence: StageEvidenceIndex,
    pub sandbox_universe: SandboxUniverse,
    pub campaign_rotations: CampaignRotations,
    pub consts: GameDataConst,
    pub missions: MissionData,
    pub profession_names: HashMap<String, String>,
    pub factions: HashMap<String, HandbookTeam>,
    pub home_backgrounds: Vec<HomeBackground>,
    pub autochess_bonds: HashMap<String, AutoChessBond>,
    pub squad: SquadIndex,
}

impl GameModel {
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        !self.operators.is_empty() && !self.skills.is_empty()
    }
}
