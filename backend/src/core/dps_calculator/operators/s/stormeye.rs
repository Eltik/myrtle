//! DPS calculations for Stormeye
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Stormeye operator implementation
pub struct Stormeye {
    pub unit: OperatorUnit,
}

impl Stormeye {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Stormeye operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            1, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// critchance = self.talent1_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(2 * final_atk - defense, 2 * final_atk * 0.05)
    /// avgdmg = critchance * critdmg + (1-critchance) * hitdmg
    /// dps = (1+self.skill) * avgdmg / self.atk_interval * (self.attack_speed) / 100
    /// return dps * min(1+self.skill, self.targets)
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

        let mut critchance = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut critdmg = ((2.0 * final_atk - defense) as f64).max((2.0 * final_atk * 0.05) as f64);
        let mut avgdmg = critchance * critdmg + (1.0 - critchance) * hitdmg;
        let mut dps = (1.0 + (self.unit.skill_index as f64)) * avgdmg
            / (self.unit.attack_interval as f64)
            * (self.unit.attack_speed)
            / 100.0;
        return dps
            * ((1.0 + (self.unit.skill_index as f64)) as f64)
                .min((self.unit.targets as f64) as f64);
    }
}

impl std::ops::Deref for Stormeye {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Stormeye {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
