use std::collections::HashMap;

use crate::core::local::types::{
    gacha::GachaData, handbook::Handbook, material::Materials, module::Modules, operator::Operator,
    range::Ranges, skill::Skill, skin::SkinData, trust::Favor, voice::Voices,
};

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
pub mod trust;
pub mod voice;

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
}

impl GameData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_loaded(&self) -> bool {
        !self.operators.is_empty() && !self.skills.is_empty()
    }
}
