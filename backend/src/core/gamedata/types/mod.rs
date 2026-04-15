use std::collections::HashMap;

pub mod activity;
pub mod building;
pub mod chibi;
pub mod enemy;
pub mod gacha;
pub mod handbook;
pub mod material;
pub mod medal;
pub mod module;
pub mod operator;
pub mod range;
pub mod roguelike;
pub mod sandbox_universe;
pub mod serde_helpers;
pub mod skill;
pub mod skin;
pub mod stage;
pub mod stage_universe;
pub mod trust;
pub mod voice;
pub mod zone;

use building::BuildingDataFile;
use chibi::ChibiData;
use enemy::EnemyHandbook;
use gacha::GachaData;
use handbook::Handbook;
use material::Materials;
use medal::MedalData;
use module::Modules;
use operator::Operator;
use range::Ranges;
use roguelike::RoguelikeGameData;
use sandbox_universe::SandboxUniverse;
use skill::Skill;
use skin::SkinData;
use stage::Stage;
use stage_universe::StageUniverse;
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
    pub zones: HashMap<String, Zone>,
    pub stages: HashMap<String, Stage>,
    pub medals: MedalData,
    pub roguelike: RoguelikeGameData,
    pub enemies: EnemyHandbook,
    pub building: BuildingDataFile,
    pub stage_universe: StageUniverse,
    pub sandbox_universe: SandboxUniverse,
}

impl GameData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_loaded(&self) -> bool {
        !self.operators.is_empty() && !self.skills.is_empty()
    }
}
