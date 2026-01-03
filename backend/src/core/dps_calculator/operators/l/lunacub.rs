//! DPS calculations for Lunacub
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Lunacub operator implementation
pub struct Lunacub {
    pub unit: OperatorUnit,
}

impl Lunacub {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Lunacub operator
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
    /// atk_shorter = 0.15 if self.elite == 2 else 0
    /// if self.module == 2: atk_shorter += 0.05 * (self.module_lvl - 1)
    /// if self.skill == 0: atk_shorter = 0
    /// atk_interval = self.atk_interval * (1-atk_shorter)
    /// atkbuff = self.skill_params[0] if self.skill == 1 else 0
    /// aspd = self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut atk_shorter = if ((self.unit.elite as f64) as f64) == 2.0 { 0.15 } else { 0.0 };
        // UNTRANSLATED: if self.module == 2: atk_shorter += 0.05 * (self.module_lvl - 1)
        if (self.unit.skill_index as f64) == 0.0 { atk_shorter = 0.0; }
        let mut atk_interval = (self.unit.attack_interval as f64) * (1.0 -atk_shorter);
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[0] } else { 0.0 };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 { self.unit.skill_parameters[0] } else { 0.0 };
        let mut final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut dps = hitdmg/atk_interval * (self.unit.attack_speed+aspd)/ 100.0;
        return dps;
    }
}

impl std::ops::Deref for Lunacub {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Lunacub {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
