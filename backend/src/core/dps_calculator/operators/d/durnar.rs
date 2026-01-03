//! DPS calculations for Durnar
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Durnar operator implementation
pub struct Durnar {
    pub unit: OperatorUnit,
}

impl Durnar {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Durnar operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// extra_scale = 0.1 if self.module == 1 else 0
    /// final_atk = self.atk * (1 + self.skill_params[0] * min(self.skill,1) + self.buff_atk + self.talent1_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1 + extra_scale) * (1-res/100), final_atk * (1 + extra_scale) * 0.05) if self.skill > 0 else np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100
    /// if self.skill == 2: dps *= min(self.targets,3)
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut extra_scale = if ((self.unit.module_index as f64) as f64) == 1.0 { 0.1 } else { 0.0 };
        let mut final_atk = self.unit.atk * (1.0 + self.unit.skill_parameters[0] * (((self.unit.skill_index as f64)) as f64).min((1) as f64) + self.unit.buff_atk + self.unit.talent1_parameters[0]) + self.unit.buff_atk_flat;
        let mut hitdmg = if ((self.unit.skill_index as f64) as f64) > 0.0 { ((final_atk * (1.0 + extra_scale) * (1.0 -res/ 100.0)) as f64).max((final_atk * (1.0 + extra_scale) * 0.05) as f64) } else { ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) };
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        // UNTRANSLATED: if self.skill == 2: dps *= min(self.targets,3)
        return dps;
    }
}

impl std::ops::Deref for Durnar {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Durnar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
