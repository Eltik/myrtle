//! DPS calculations for Rockrock
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Rockrock operator implementation
pub struct Rockrock {
    pub unit: OperatorUnit,
}

impl Rockrock {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Rockrock operator
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
    /// drone_dmg = 1.1
    /// if not self.trait_dmg:
    /// drone_dmg = 0.35 if self.module == 1 else 0.2
    /// atkbuff = self.talent1_params[0] * self.talent1_params[1] if self.talent_dmg and self.elite > 0 else 0
    /// aspd = 5 if self.module == 1 and self.module_lvl == 3 and self.talent_dmg else 0
    /// aspd += self.skill_params[1] if self.skill == 2 else self.skill_params[0] * self.skill
    /// if self.skill_dmg and self.skill == 2: atkbuff += self.skill_params[0]
    /// if self.skill == 2 and self.skill_dmg and self.trait_dmg: drone_dmg *= self.skill_params[3]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// dmgperinterval = final_atk + drone_dmg * final_atk
    /// hitdmgarts = np.fmax(dmgperinterval *(1-res/100), dmgperinterval * 0.05)
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100
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

        let mut drone_dmg = 1.1;
        if !self.unit.trait_damage {
            drone_dmg = if ((self.unit.module_index as f64) as f64) == 1.0 {
                0.35
            } else {
                0.2
            };
        }
        let mut atkbuff = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
                * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut aspd = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) == 3.0
            && self.unit.talent_damage
        {
            5.0
        } else {
            0.0
        };
        aspd += if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * ((self.unit.skill_index as f64) as f64)
        };
        if self.unit.skill_damage && (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
        }
        if (self.unit.skill_index as f64) == 2.0 && self.unit.skill_damage && self.unit.trait_damage
        {
            drone_dmg *= self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
        }
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut dmgperinterval = final_atk + drone_dmg * final_atk;
        let mut hitdmgarts =
            ((dmgperinterval * (1.0 - res / 100.0)) as f64).max((dmgperinterval * 0.05) as f64);
        let mut dps = hitdmgarts / (self.unit.attack_interval as f64)
            * (self.unit.attack_speed + aspd)
            / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Rockrock {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Rockrock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
