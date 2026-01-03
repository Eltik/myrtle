//! DPS calculations for Lee
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Lee operator implementation
pub struct Lee {
    pub unit: OperatorUnit,
}

impl Lee {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Lee operator
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
    /// aspd = self.talent1_params[1] if self.talent_dmg and self.elite > 0 else 0
    /// if self.targets == 1 and self.talent_dmg: aspd *= 2
    /// atkbuff = 0.2 if self.module == 2 and self.module_dmg else 0
    /// if self.skill == 2: aspd += self.skill_params[5]
    /// else: atkbuff += self.skill_params[0] * min(self.skill,1)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
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

        let mut aspd = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters[1]
        } else {
            0.0
        };
        // UNTRANSLATED: if self.targets == 1 and self.talent_dmg: aspd *= 2
        let mut atkbuff =
            if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
                0.2
            } else {
                0.0
            };
        // UNTRANSLATED: if self.skill == 2: aspd += self.skill_params[5]
        // UNTRANSLATED: else: atkbuff += self.skill_params[0] * min(self.skill,1)
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut dps =
            hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Lee {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Lee {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
