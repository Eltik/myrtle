//! DPS calculations for KroosAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// KroosAlter operator implementation
pub struct KroosAlter {
    pub unit: OperatorUnit,
}

impl KroosAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new KroosAlter operator
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
    /// crate = 0 if self.elite == 0 else self.talent1_params[0]
    /// cdmg = self.talent1_params[1]
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// atkbuff = self.skill_params[0] if self.skill == 1 else 0
    /// atk_interval = self.atk_interval * (1 + self.skill_params[0]) if self.skill == 2 else self.atk_interval
    /// hits = 4 if self.skill == 2 and self.skill_dmg else 1 + min(self.skill, 1)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale -defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale * cdmg -defense, final_atk * atk_scale * cdmg * 0.05)
    /// avgdmg = critdmg * crate + hitdmg * (1-crate)
    /// dps = hits * avgdmg/atk_interval * self.attack_speed/100
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

        let mut crit_rate = if ((self.unit.elite as f64) as f64) == 0.0 {
            0.0
        } else {
            self.unit.talent1_parameters[0]
        };
        let mut cdmg = self.unit.talent1_parameters[1];
        let mut atk_scale =
            if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
                1.1
            } else {
                1.0
            };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters[0]
        } else {
            0.0
        };
        let mut atk_interval = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            (self.unit.attack_interval as f64) * (1.0 + self.unit.skill_parameters[0])
        } else {
            (self.unit.attack_interval as f64)
        };
        let mut hits = if ((self.unit.skill_index as f64) as f64) == 2.0 && self.unit.skill_damage {
            4.0
        } else {
            1.0 + (((self.unit.skill_index as f64) as f64) as f64).min((1) as f64)
        };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut critdmg = ((final_atk * atk_scale * cdmg - defense) as f64)
            .max((final_atk * atk_scale * cdmg * 0.05) as f64);
        let mut avgdmg = critdmg * crit_rate + hitdmg * (1.0 - crit_rate);
        let mut dps = hits * avgdmg / atk_interval * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for KroosAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for KroosAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
