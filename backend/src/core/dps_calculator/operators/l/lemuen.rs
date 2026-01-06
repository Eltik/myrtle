//! DPS calculations for Lemuen
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Lemuen operator implementation
pub struct Lemuen {
    pub unit: OperatorUnit,
}

impl Lemuen {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("talent", "vsMarked", false, &[], &[], 1, 0)];

    /// Creates a new Lemuen operator
    #[allow(unused_parens)]
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
    /// atkbuff = self.talent2_params[1] if self.talent2_dmg and self.elite > 1 else 0
    /// dmg = min(self.talent1_params) if self.talent_dmg and self.elite > 0 else 1
    ///
    /// if self.skill < 2:
    /// atk_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * min(self.targets, 1 + self.skill) * dmg
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 2:
    /// aspd = self.skill_params[0]
    /// atkbuff += self.skill_params[1]
    /// atk_scale = self.skill_params[8]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// if self.talent_dmg: hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * dmg
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.talent_dmg:
    /// dps = hitdmg / self.skill_params[3]
    /// if self.skill == 3:
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// ammo = 5 + self.talent2_params[2] if self.talent2_dmg else 5
    /// centralhit_dmg = np.fmax(final_atk * self.skill_params[4] - defense, final_atk * self.skill_params[4] * 0.05) * dmg
    /// outerhit_dmg = np.fmax(final_atk * self.skill_params[6] - defense, final_atk * self.skill_params[6] * 0.05) * dmg
    /// dps = ammo * centralhit_dmg * self.targets
    ///
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        atkbuff = if self.unit.talent2_damage && ((self.unit.elite as f64) as f64) > 1.0 {
            self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut dmg = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit
                .talent1_parameters
                .iter()
                .cloned()
                .fold(f64::INFINITY, f64::min)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atk_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * ((self.unit.targets as f64) as f64)
                    .min((1.0 + (self.unit.skill_index as f64)) as f64)
                * dmg;
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            aspd = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            atkbuff += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            atk_scale = self.unit.skill_parameters.get(8).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            if self.unit.talent_damage {
                hitdmg = ((final_atk * atk_scale - defense) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64)
                    * dmg;
            }
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
            if self.unit.talent_damage {
                dps = hitdmg / self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            let mut ammo = if self.unit.talent2_damage {
                5.0 + self.unit.talent2_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                5.0
            };
            let mut centralhit_dmg = ((final_atk
                * self.unit.skill_parameters.get(4).copied().unwrap_or(0.0)
                - defense) as f64)
                .max(
                    (final_atk * self.unit.skill_parameters.get(4).copied().unwrap_or(0.0) * 0.05)
                        as f64,
                )
                * dmg;
            let mut outerhit_dmg = ((final_atk
                * self.unit.skill_parameters.get(6).copied().unwrap_or(0.0)
                - defense) as f64)
                .max(
                    (final_atk * self.unit.skill_parameters.get(6).copied().unwrap_or(0.0) * 0.05)
                        as f64,
                )
                * dmg;
            dps = ammo * centralhit_dmg * (self.unit.targets as f64);
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// extra_ammo = self.talent2_params[2] if self.elite > 1 and self.talent2_dmg else 0
    /// if self.skill == 1:
    /// return(self.skill_dps(defense,res) * (self.skill_params[1] + extra_ammo) * (self.atk_interval/(self.attack_speed/100)))
    /// elif self.skill == 2 and self.talent_dmg:
    /// return(self.skill_dps(defense,res) * (self.skill_params[2] + extra_ammo) * (self.skill_params[3]))
    /// else:
    /// return(super().total_dmg(defense,res))
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut extra_ammo = if ((self.unit.elite as f64) as f64) > 1.0 && self.unit.talent2_damage
        {
            self.unit.talent2_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * (self.skill_params[1] + extra_ammo) * (self.atk_interval/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else if (self.unit.skill_index as f64) == 2.0 && self.unit.talent_damage {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * (self.skill_params[2] + extra_ammo) * (self.skill_params[3])) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(super().total_dmg(defense,res)) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for Lemuen {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Lemuen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Lemuen {
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
