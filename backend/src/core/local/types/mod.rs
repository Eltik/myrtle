use std::collections::HashMap;

use crate::core::local::{
    asset_mapping::AssetMappings,
    types::{
        chibi::ChibiData, gacha::GachaData, handbook::Handbook, material::Materials,
        module::Modules, operator::Operator, range::Ranges, skill::Skill, skin::SkinData,
        stage::Stage, trust::Favor, voice::Voices, zone::Zone,
    },
};

pub mod chibi;
pub mod enemy;
pub mod gacha;
pub mod handbook;
pub mod material;
pub mod module;
pub mod operator;
pub mod range;
pub mod serde_helpers;
pub mod skill;
pub mod skin;
pub mod stage;
pub mod trust;
pub mod voice;
pub mod zone;

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
    pub asset_mappings: AssetMappings,
}

impl GameData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_loaded(&self) -> bool {
        !self.operators.is_empty() && !self.skills.is_empty()
    }
}
