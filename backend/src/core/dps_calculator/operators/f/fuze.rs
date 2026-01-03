//! DPS calculations for Fuze
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Fuze operator implementation
pub struct Fuze {
    pub unit: OperatorUnit,
}

impl Fuze {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Fuze operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atkbuff = self.skill_params[1] if self.skill == 1 else 0
    /// aspd = self.skill_params[2] if self.skill == 1 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// max_targets = 3 if self.elite == 2 else 2
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * min(self.targets,max_targets)
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

        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters[1]
        } else {
            0.0
        };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters[2]
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut max_targets = if ((self.unit.elite as f64) as f64) == 2.0 {
            3.0
        } else {
            2.0
        };
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0
            * ((self.unit.targets as f64) as f64).min((max_targets) as f64);
        return dps;
    }
}

impl std::ops::Deref for Fuze {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Fuze {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
