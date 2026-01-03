//! DPS calculations for April
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// April operator implementation
pub struct April {
    pub unit: OperatorUnit,
}

impl April {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new April operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// aspd = 8 if self.module == 2 and self.module_dmg else 0
    /// if self.skill == 1:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * self.skill_params[0] - defense, final_atk * self.skill_params[0] * 0.05)
    /// avgdmg = (self.skill_cost * hitdmg + skilldmg) / (self.skill_cost + 1)
    /// dps = avgdmg / self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill != 1:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill/2) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;

        aspd = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage { 8.0 } else { 0.0 };
        if (self.unit.skill_index as f64) == 1.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        skilldmg = ((final_atk * self.unit.skill_parameters[0] - defense) as f64).max((final_atk * self.unit.skill_parameters[0] * 0.05) as f64);
        avgdmg = ((self.unit.skill_cost as f64) * hitdmg + skilldmg) / ((self.unit.skill_cost as f64) + 1.0);
        dps = avgdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) != 1.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[0] * (self.unit.skill_index as f64)/ 2.0) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for April {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for April {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
