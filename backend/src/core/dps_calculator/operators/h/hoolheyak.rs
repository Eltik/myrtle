//! DPS calculations for Hoolheyak
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Hoolheyak operator implementation
pub struct Hoolheyak {
    pub unit: OperatorUnit,
}

impl Hoolheyak {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "vsAerial", false, &[], &[], 0, 0),
        ("skill", "maxRange", false, &[], &[], 0, 0),
        ("talent2", "vsLowHp", false, &[], &[2], 0, 2),
        ("module", "vsElite", false, &[1], &[2], 0, 0),
    ];

    /// Creates a new Hoolheyak operator
    #[allow(unused_parens)]
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
    /// atk_scale = self.talent1_params[0] if self.talent_dmg and self.elite > 0 else 1
    /// newres = np.fmax(res-10,0) if self.module in [1,3] else res
    /// dmg_scale = 1
    /// if self.module == 2 and self.talent2_dmg:
    /// dmg_scale += 0.1 * (self.module_lvl -1)
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// sp_cost = self.skill_cost/(1 + self.sp_boost) + 1.2 #sp lockout
    /// if self.module == 2 and self.module_dmg: sp_cost = self.skill_cost/(1 + self.sp_boost + 1/self.atk_interval*self.attack_speed/100) + 1.2 #sp lockout
    /// hitdmgarts = np.fmax(final_atk * atk_scale * (1-newres/100), final_atk * atk_scale * 0.05) * dmg_scale
    /// skilldmg = np.fmax(final_atk * atk_scale * skill_scale * (1-newres/100), final_atk * atk_scale * skill_scale * 0.05) * dmg_scale
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg * min(2, self.targets)
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[2] > 1:
    /// avghit = (skilldmg * min(2, self.targets) + (atks_per_skillactivation - 1) * hitdmgarts) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg * min(2, self.targets) + int(atks_per_skillactivation) * hitdmgarts) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// hitdmgarts = np.fmax(final_atk * atk_scale * skill_scale * (1-newres/100), final_atk * atk_scale * skill_scale * 0.05) * dmg_scale
    /// dps = 9 * hitdmgarts/self.atk_interval * self.attack_speed/100
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[1] if self.skill_dmg else self.skill_params[0]
    /// hitdmgarts = np.fmax(final_atk * atk_scale * skill_scale * (1-newres/100), final_atk * atk_scale * skill_scale * 0.05) * dmg_scale
    /// dps = hitdmgarts/3 * self.attack_speed/100 * min(self.targets, 3)
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

        let mut sp_cost: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut avghit: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        atk_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        newres = if [1.0, 3.0].contains(&(((self.unit.module_index as f64) as f64) as f64)) {
            ((res - 10.0) as f64).max((0) as f64)
        } else {
            res
        };
        let mut dmg_scale = 1.0;
        if (self.unit.module_index as f64) == 2.0 && self.unit.talent2_damage {
            dmg_scale += 0.1 * ((self.unit.module_level as f64) - 1.0);
        }
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            if (self.unit.module_index as f64) == 2.0 && self.unit.module_damage {
                sp_cost = (self.unit.skill_cost as f64)
                    / (1.0
                        + (self.unit.sp_boost as f64)
                        + 1.0 / (self.unit.attack_interval as f64) * self.unit.attack_speed
                            / 100.0)
                    + 1.2;
            }
            hitdmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg_scale;
            skilldmg = ((final_atk * atk_scale * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64)
                * dmg_scale;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg * ((2) as f64).min((self.unit.targets as f64) as f64);
            if atks_per_skillactivation > 1.0 {
                if self.unit.skill_parameters[2] > 1.0 {
                    avghit = (skilldmg * ((2) as f64).min((self.unit.targets as f64) as f64)
                        + (atks_per_skillactivation - 1.0) * hitdmgarts)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg * ((2) as f64).min((self.unit.targets as f64) as f64)
                        + ((atks_per_skillactivation) as f64).trunc() * hitdmgarts)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            hitdmgarts = ((final_atk * atk_scale * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64)
                * dmg_scale;
            dps = 9.0 * hitdmgarts / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = if self.unit.skill_damage {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            };
            hitdmgarts = ((final_atk * atk_scale * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64)
                * dmg_scale;
            dps = hitdmgarts / 3.0 * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Hoolheyak {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Hoolheyak {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Hoolheyak {
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
