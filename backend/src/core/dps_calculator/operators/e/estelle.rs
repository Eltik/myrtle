//! DPS calculations for Estelle
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Estelle operator implementation
pub struct Estelle {
    pub unit: OperatorUnit,
}

impl Estelle {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Estelle operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * min(1,self.skill)) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// block = 3 if self.elite == 2 else 2
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * min(self.targets,block)
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[0] * ((1) as f64).min(((self.unit.skill_index as f64)) as f64)) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut block = if ((self.unit.elite as f64) as f64) == 2.0 { 3.0 } else { 2.0 };
        let mut dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 * (((self.unit.targets as f64)) as f64).min((block) as f64);
        return dps;
    }
}

impl std::ops::Deref for Estelle {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Estelle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
