//! DPS calculations for MisumiUika
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// MisumiUika operator implementation
pub struct MisumiUika {
    pub unit: OperatorUnit,
}

impl MisumiUika {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new MisumiUika operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[2] + self.talent2_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// dps = hitdmg / 0.3
    /// else:
    /// dps = res * 0
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        if (self.unit.skill_index as f64) == 2.0 {
        skill_scale = self.unit.skill_parameters[2] + self.unit.talent2_parameters[1];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * skill_scale * (1.0 -res/ 100.0)) as f64).max((final_atk * skill_scale * 0.05) as f64);
        dps = hitdmg / 0.3;
        } else {
        dps = res * 0.0;
        }
        return dps;
    }
}

impl std::ops::Deref for MisumiUika {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for MisumiUika {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
