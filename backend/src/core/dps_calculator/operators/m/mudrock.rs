//! DPS calculations for Mudrock
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Mudrock operator implementation
pub struct Mudrock {
    pub unit: OperatorUnit,
}

impl Mudrock {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Mudrock operator
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
    /// atkbuff = self.skill_params[0] if self.skill == 3 else 0
    /// if self.module == 2 and self.module_dmg: atkbuff += 0.08
    /// dmg = self.talent2_params[1] if self.module == 2 and self.module_lvl > 1 and self.talent2_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// atk_interval = self.atk_interval * 0.7 if self.skill == 3 else self.atk_interval
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * dmg
    /// dps = hitdmg/atk_interval * self.attack_speed/100
    /// if self.skill == 3: dps *= min(self.targets,3)
    ///
    /// if self.skill == 2 and self.hits > 0:
    /// atk_scale = self.skill_params[0]
    /// skilldmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * dmg
    /// spcost = self.skill_cost
    /// extra_sp = (self.module_lvl-1)/9 if self.module == 1 else 0
    /// if self.module_lvl == 2: extra_sp *= (spcost-1)/spcost #these roughly factor in the wasted potential. realistically more gets wasted due to the lockout
    /// if self.module_lvl == 3: extra_sp *= (2*spcost-3)/(2*spcost)
    /// skillcycle = spcost / (self.hits+extra_sp) + 1.2
    /// dps += skilldmg / skillcycle * self.targets
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
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        atkbuff = if ((self.unit.skill_index as f64) as f64) == 3.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.module_index as f64) == 2.0 && self.unit.module_damage {
            atkbuff += 0.08;
        }
        let mut dmg = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && self.unit.talent2_damage
        {
            self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        atk_interval = if ((self.unit.skill_index as f64) as f64) == 3.0 {
            (self.unit.attack_interval as f64) * 0.7
        } else {
            (self.unit.attack_interval as f64)
        };
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * dmg;
        dps = hitdmg / atk_interval * self.unit.attack_speed / 100.0;
        if (self.unit.skill_index as f64) == 3.0 {
            dps *= ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 && 0.0 /* self.hits - defaults to 0 */ > 0.0 {
            atk_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            skilldmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg;
            let mut spcost = (self.unit.skill_cost as f64);
            let mut extra_sp = if ((self.unit.module_index as f64) as f64) == 1.0 {
                (((self.unit.module_level as f64) as f64) - 1.0) / 9.0
            } else {
                0.0
            };
            if (self.unit.module_level as f64) == 2.0 {
                extra_sp *= (spcost - 1.0) / spcost;
            }
            if (self.unit.module_level as f64) == 3.0 {
                extra_sp *= (2.0 * spcost - 3.0) / (2.0 * spcost);
            }
            let mut skillcycle = spcost / (0.0 /* self.hits - defaults to 0 */+extra_sp) + 1.2;
            dps += skilldmg / skillcycle * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Mudrock {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Mudrock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Mudrock {
    fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        Self::skill_dps(self, enemy)
    }

    fn unit(&self) -> &OperatorUnit {
        &self.unit
    }

    fn unit_mut(&mut self) -> &mut OperatorUnit {
        &mut self.unit
    }
}
