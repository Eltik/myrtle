//! DPS calculations for Ray
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Ray operator implementation
pub struct Ray {
    pub unit: OperatorUnit,
}

impl Ray {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[
        ("trait", "outOfAmmo", true, &[], &[], 0, 0),
        ("talent", "with pet", false, &[], &[], 1, 0),
        ("talent2", "After3Hits", false, &[], &[], 2, 0),
    ];

    /// Creates a new Ray operator
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
    /// atk_scale = 1.2
    /// dmg_scale = 1 + self.talent1_params[0] if self.talent_dmg and self.elite > 0 else 1
    /// atkbuff = self.talent2_params[0] * self.talent2_params[1] if self.talent2_dmg and self.elite == 2 else 0
    ///
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05) * dmg_scale
    /// skilldmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05) * dmg_scale
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if not self.trait_dmg:
    /// dps = hitdmg/(self.atk_interval * self.attack_speed/100 + 1.6)
    /// if self.module == 1: dps = 2*hitdmg/(2 * self.atk_interval * self.attack_speed/100 + 1.6)
    /// dps += skilldmg /(self.skill_cost/(1+self.sp_boost)+1.2)
    /// if self.skill in [0,2]:
    /// atkbuff += self.skill_params[0] * self.skill / 2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk *atk_scale - defense, final_atk* atk_scale * 0.05) * dmg_scale
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if not self.trait_dmg:
    /// dps = hitdmg/(self.atk_interval * self.attack_speed/100 + 1.6)
    /// if self.module == 1: dps = 2*hitdmg/(2 * self.atk_interval * self.attack_speed/100 + 1.6)
    /// if self.skill == 3:
    /// atk_scale *= self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * dmg_scale
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if not self.trait_dmg:
    /// dps = hitdmg/(self.atk_interval * self.attack_speed/100 + 0.4)
    /// if self.module == 1: dps = 2*hitdmg/(2 * self.atk_interval * self.attack_speed/100 + 0.4)
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

        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        atk_scale = 1.2;
        let mut dmg_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            1.0 + self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        atkbuff = if self.unit.talent2_damage && ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
                * self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg_scale;
            skilldmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64)
                * dmg_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if !self.unit.trait_damage {
                dps = hitdmg
                    / ((self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0 + 1.6);
                if (self.unit.module_index as f64) == 1.0 {
                    dps = 2.0 * hitdmg
                        / (2.0 * (self.unit.attack_interval as f64) * self.unit.attack_speed
                            / 100.0
                            + 1.6);
                }
            }
            dps += skilldmg
                / ((self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2);
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64)
                / 2.0;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if !self.unit.trait_damage {
                dps = hitdmg
                    / ((self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0 + 1.6);
                if (self.unit.module_index as f64) == 1.0 {
                    dps = 2.0 * hitdmg
                        / (2.0 * (self.unit.attack_interval as f64) * self.unit.attack_speed
                            / 100.0
                            + 1.6);
                }
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_scale *= self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if !self.unit.trait_damage {
                dps = hitdmg
                    / ((self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0 + 0.4);
                if (self.unit.module_index as f64) == 1.0 {
                    dps = 2.0 * hitdmg
                        / (2.0 * (self.unit.attack_interval as f64) * self.unit.attack_speed
                            / 100.0
                            + 0.4);
                }
            }
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// if self.skill == 3:
    /// return(self.skill_dps(defense,res) * 8 * (self.atk_interval/(self.attack_speed/100)))
    /// else:
    /// return(self.skill_dps(defense,res))
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
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        if (self.unit.skill_index as f64) == 3.0 {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * 8 * (self.atk_interval/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(self.skill_dps(defense,res)) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for Ray {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ray {
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
