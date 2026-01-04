//! DPS calculations for Steward
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Steward operator implementation
pub struct Steward {
    pub unit: OperatorUnit,
}

impl Steward {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Steward operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// final_atk = self.atk * (1 + self.talent1_params[0] + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmg_skill = np.fmax(final_atk * self.skill_params[0] * (1-res/100), final_atk * self.skill_params[0] * 0.05)
    /// avghit = (hitdmg * self.skill_cost + hitdmg_skill) / (self.skill_cost + 1)
    /// dps = avghit / self.atk_interval * self.attack_speed/100
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens,
        clippy::if_same_then_else,
        clippy::nonminimal_bool,
        clippy::overly_complex_bool_expr,
        clippy::needless_return,
        clippy::collapsible_else_if,
        clippy::neg_multiply,
        clippy::assign_op_pattern,
        clippy::eq_op
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut final_atk = self.unit.atk
            * (1.0
                + self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
                + self.unit.buff_atk)
            + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut hitdmg_skill = ((final_atk
            * self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            * (1.0 - res / 100.0)) as f64)
            .max(
                (final_atk * self.unit.skill_parameters.first().copied().unwrap_or(0.0) * 0.05)
                    as f64,
            );
        let mut avghit = (hitdmg * (self.unit.skill_cost as f64) + hitdmg_skill)
            / ((self.unit.skill_cost as f64) + 1.0);
        let mut dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Steward {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Steward {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
