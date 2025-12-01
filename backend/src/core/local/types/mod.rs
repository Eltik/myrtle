use std::collections::HashMap;

use crate::core::local::types::{operator::Operator, skill::Skill};

pub mod handbook;
pub mod material;
pub mod module;
pub mod operator;
pub mod skill;
pub mod skin;
pub mod voice;

#[derive(Debug, Clone, Default)]
pub struct GameData {
    pub operators: HashMap<String, Operator>,
    pub skills: HashMap<String, Skill>,
    // TODO: Etc.
}

impl GameData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_loaded(&self) -> bool {
        // TODO: Add as GameData is populated
        !self.operators.is_empty() && !self.skills.is_empty()
    }
}
