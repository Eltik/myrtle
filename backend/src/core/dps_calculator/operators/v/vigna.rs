//! DPS calculations for Vigna
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Vigna operator implementation
pub struct Vigna {
    pub unit: OperatorUnit,
}

impl Vigna {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Vigna operator
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
    /// crate = 0 if self.elite == 0 else self.talent1_params[2]
    /// if self.skill == 0 and self.elite > 0: crate = self.talent1_params[1]
    /// cdmg = self.talent1_params[0]
    /// atkbuff = self.skill_params[0] * min(self.skill, 1)
    /// atk_interval = 1.5 if self.skill == 2 else self.atk_interval
    /// atk_scale = 1.1 if self.module == 2 and self.module_dmg else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// final_atk_crit = self.atk * (1 + atkbuff + self.buff_atk + cdmg) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk_crit * atk_scale - defense, final_atk_crit * atk_scale * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg / atk_interval * self.attack_speed/100
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
            self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        };
        if (self.unit.skill_index as f64) == 0.0 && (self.unit.elite as f64) > 0.0 {
            crit_rate = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        }
        let mut cdmg = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        let mut atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            * ((self.unit.skill_index as f64) as f64).min((1) as f64);
        let mut atk_interval = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            1.5
        } else {
            (self.unit.attack_interval as f64)
        };
        let mut atk_scale =
            if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
                1.1
            } else {
                1.0
            };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut final_atk_crit =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk + cdmg) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut critdmg = ((final_atk_crit * atk_scale - defense) as f64)
            .max((final_atk_crit * atk_scale * 0.05) as f64);
        let mut avgdmg = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
        let mut dps = avgdmg / atk_interval * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Vigna {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Vigna {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
