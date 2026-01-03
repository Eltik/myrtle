//! DPS calculations for Lin
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Lin operator implementation
pub struct Lin {
    pub unit: OperatorUnit,
}

impl Lin {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Lin operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
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
    /// if self.skill == 0: return res * 0
    /// if self.skill == 2:
    /// aspd = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// else:
    /// if self.skill == 1: self.atk_interval = 3
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmgarts/self.atk_interval * self.attack_speed/100 * self.targets
    /// if self.module == 2 and self.module_dmg: dps *= 1.15
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        // UNTRANSLATED: if self.skill == 0: return res * 0
        if (self.unit.skill_index as f64) == 2.0 {
        aspd = self.unit.skill_parameters[0];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk * (1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmgarts/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0 * (self.unit.targets as f64);
        } else {
        // UNTRANSLATED: if self.skill == 1: self.atk_interval = 3
        atkbuff = self.unit.skill_parameters[0];
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk * (1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmgarts/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 * (self.unit.targets as f64);
        }
        // UNTRANSLATED: if self.module == 2 and self.module_dmg: dps *= 1.15
        return dps;
    }
}

impl std::ops::Deref for Lin {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Lin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
