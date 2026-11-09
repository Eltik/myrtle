use std::collections::HashMap;

use myrtle_gamedata::model::stage_universe::StageUniverse;
use myrtle_gamedata_raw::tables::{
    medal::MedalData, operator::Operator, roguelike::RoguelikeGameData,
    sandbox_universe::SandboxUniverse, trust::Favor,
};

pub mod inputs;
pub mod medals;
pub mod operators;
pub mod roguelike;
pub mod total;

pub mod sandbox;
pub mod stages;

pub use total as calculate;

pub use inputs::{BaseScore, GradeInputs, StageClears};
pub use total::{UserGrade, calculate_user_grade};

#[derive(Debug, Clone, Default)]
pub struct GradeData {
    pub operators: HashMap<String, Operator>,
    pub favor: Favor,
    pub roguelike: RoguelikeGameData,
    pub medals: MedalData,
    pub stage_universe: StageUniverse,
    pub sandbox_universe: SandboxUniverse,
}

pub(crate) type Dimension = (f64, f64);

pub(crate) fn weighted_average(dims: &[Dimension]) -> f64 {
    let total_weight: f64 = dims.iter().map(|(w, _)| w).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    dims.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight
}
