use std::{collections::HashMap, sync::Arc};

use myrtle_gamedata_raw::tables::{
    building::BuildingDataFile, consts::GameDataConst, operator::Operator,
};

pub mod assignment;
pub mod context;
pub mod registry;
pub mod rotation;
pub mod score;
pub mod scoring;
pub mod types;
pub(crate) mod util;

#[cfg(test)]
#[path = "../tests/unit/mod.rs"]
mod unit;

pub use context::BaseContext;
use registry::{BuffResolutionStrategy, build_name_to_char, build_registry, glossary_tags};
pub use score::{BaseGrade, grade_base};
pub use util::{buff_family, is_production_room};

/// The tables the base engine reads and what it parses from them
#[derive(Debug, Clone)]
pub struct BaseData {
    operators: HashMap<String, Operator>,
    building: BuildingDataFile,
    consts: GameDataConst,
    glossary: HashMap<String, Vec<String>>,
    registry: Arc<HashMap<String, BuffResolutionStrategy>>,
    morale_drains: Arc<HashMap<String, f64>>,
}

impl BaseData {
    #[must_use]
    pub fn new(
        operators: HashMap<String, Operator>,
        building: BuildingDataFile,
        consts: GameDataConst,
    ) -> Self {
        let name_to_char = build_name_to_char(&operators);
        let glossary = glossary_tags(&consts, &name_to_char);
        let (registry, morale_drains) = build_registry(&building.buffs, &name_to_char);
        Self {
            operators,
            building,
            consts,
            glossary,
            registry: Arc::new(registry),
            morale_drains: Arc::new(morale_drains),
        }
    }

    #[must_use]
    pub const fn operators(&self) -> &HashMap<String, Operator> {
        &self.operators
    }

    #[must_use]
    pub const fn building(&self) -> &BuildingDataFile {
        &self.building
    }

    #[must_use]
    pub const fn consts(&self) -> &GameDataConst {
        &self.consts
    }

    /// `char_id` -> glossary tags
    #[must_use]
    pub const fn glossary(&self) -> &HashMap<String, Vec<String>> {
        &self.glossary
    }

    #[must_use]
    pub const fn registry(&self) -> &Arc<HashMap<String, BuffResolutionStrategy>> {
        &self.registry
    }

    /// `buff_id` -> morale drain modifier
    #[must_use]
    pub const fn morale_drains(&self) -> &Arc<HashMap<String, f64>> {
        &self.morale_drains
    }
}
