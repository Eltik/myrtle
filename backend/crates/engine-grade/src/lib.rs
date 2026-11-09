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

#[cfg(test)]
mod tests {
    use super::weighted_average;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn averages_by_weight() {
        assert!(close(weighted_average(&[(30.0, 1.0), (10.0, 0.0)]), 0.75));
        assert!(close(weighted_average(&[(1.0, 0.2), (1.0, 0.4)]), 0.3));
    }

    #[test]
    fn a_single_dimension_is_its_own_score() {
        assert!(close(weighted_average(&[(25.0, 0.6)]), 0.6));
    }

    #[test]
    fn non_positive_total_weight_reads_zero() {
        assert!(close(weighted_average(&[]), 0.0));
        assert!(close(weighted_average(&[(0.0, 1.0)]), 0.0));
        assert!(close(weighted_average(&[(-2.0, 1.0), (1.0, 1.0)]), 0.0));
    }
}
