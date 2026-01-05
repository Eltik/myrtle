//! DPS calculations for Necrass
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Necrass operator implementation
pub struct Necrass {
    pub unit: OperatorUnit,
}

impl Necrass {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Necrass operator
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
    /// dmg_scale = self.talent2_params[1] if self.elite > 1 and self.talent2_dmg else 1
    /// atk_scale = 1.15 if self.module_dmg and self.module == 1 else 1
    /// atkbuff = 0.05 + 0.1 * self.module_lvl if self.module == 1 and self.module_lvl > 1  and not self.no_kill else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// summon_atk = self.talent1_params[2] if self.talent_dmg else 0
    /// if self.skill == 0:
    /// drones = self.talent1_params[0] if self.trait_dmg else 0
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg_scale * atk_scale
    /// dps = hitdmg / self.atk_interval * (self.attack_speed) / 100
    /// final_atk_summon = self.drone_atk * (1 + self.buff_atk + summon_atk) + self.buff_atk_flat
    /// summondmg = np.fmax(final_atk_summon * (1-res/100), final_atk_summon * 0.05) * dmg_scale
    /// dps += drones * summondmg / self.drone_atk_interval * (self.attack_speed) / 100
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg_scale * atk_scale
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_scale
    /// dps = hitdmg / self.atk_interval * (self.attack_speed) / 100
    /// hits = self.talent1_params[0] if self.trait_dmg else 1
    /// dps += hits * skilldmg * self.targets / (self.skill_cost/(1 + self.sp_boost) + 1.2) #sp lockout
    /// final_atk_summon = self.drone_atk * (1 + self.buff_atk + summon_atk) + self.buff_atk_flat
    /// summondmg = np.fmax(final_atk_summon * (1-res/100), final_atk_summon * 0.05) * dmg_scale
    /// dps += hits * summondmg / self.drone_atk_interval * (self.attack_speed) / 100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_scale * atk_scale
    /// dps = 2 * hitdmg * min(self.targets,2)
    /// if self.skill == 3:
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg_scale * atk_scale
    /// dps = hitdmg / self.atk_interval * (self.attack_speed) / 100
    /// skill_scale = self.skill_params[1]
    /// skill_hits = self.skill_params[4]
    /// skilldmg =  np.fmax(final_atk *skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_scale * atk_scale
    /// dps += skilldmg * skill_hits * self.targets / (self.skill_cost/(1 + self.sp_boost) + 1.2 + skill_hits)
    /// if self.trait_dmg:
    /// main_atk_buff = 2 + 0.8 * 6 if self.skill_dmg else 2
    /// main_summon_atk = self.drone_atk * (1 + self.buff_atk + main_atk_buff) + self.buff_atk_flat
    /// mainhit = np.fmax(main_summon_atk * (1-res/100), main_summon_atk * 0.05) * dmg_scale
    /// maindps = mainhit/(self.drone_atk_interval + 0.7) * self.attack_speed/100
    /// mainskilldps = np.fmax(main_summon_atk * 1.2 * (1-res/100), main_summon_atk * 1.2 * 0.05) * dmg_scale
    /// dps += (maindps * 15/(1+self.sp_boost) + mainskilldps * 8 * self.targets) / (8 + 15/(1+self.sp_boost))
    /// if self.talent_dmg:
    /// final_atk_summon = self.drone_atk * (1 + self.buff_atk + summon_atk) + self.buff_atk_flat
    /// summondmg = np.fmax(final_atk_summon * (1-res/100), final_atk_summon * 0.05) * dmg_scale
    /// dps += 2 * summondmg / self.drone_atk_interval * (self.attack_speed) / 100
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
        clippy::eq_op
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut final_atk: f64 = 0.0;
        let mut final_atk_summon: f64 = 0.0;
        let mut mainhit: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut maindps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut main_atk_buff: f64 = 0.0;
        let mut mainskilldps: f64 = 0.0;
        let mut main_summon_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut summondmg: f64 = 0.0;

        let mut dmg_scale = if ((self.unit.elite as f64) as f64) > 1.0 && self.unit.talent2_damage {
            self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        atk_scale = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 1.0 {
            1.15
        } else {
            1.0
        };
        atkbuff = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && !false
        /* false /* self.no_kill - needs manual implementation */ - needs manual implementation */
        {
            0.05 + 0.1 * ((self.unit.module_level as f64) as f64)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut summon_atk = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            let mut drones = if self.unit.trait_damage {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            } else {
                0.0
            };
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale
                * atk_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
            final_atk_summon = self.unit.drone_atk * (1.0 + self.unit.buff_atk + summon_atk)
                + self.unit.buff_atk_flat;
            summondmg = ((final_atk_summon * (1.0 - res / 100.0)) as f64)
                .max((final_atk_summon * 0.05) as f64)
                * dmg_scale;
            dps += drones * summondmg / (self.unit.drone_atk_interval as f64)
                * (self.unit.attack_speed)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale
                * atk_scale;
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
            let mut hits = if self.unit.trait_damage {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            dps += hits * skilldmg * (self.unit.targets as f64)
                / ((self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2);
            final_atk_summon = self.unit.drone_atk * (1.0 + self.unit.buff_atk + summon_atk)
                + self.unit.buff_atk_flat;
            summondmg = ((final_atk_summon * (1.0 - res / 100.0)) as f64)
                .max((final_atk_summon * 0.05) as f64)
                * dmg_scale;
            dps += hits * summondmg / (self.unit.drone_atk_interval as f64)
                * (self.unit.attack_speed)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_scale
                * atk_scale;
            dps = 2.0 * hitdmg * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale
                * atk_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            let mut skill_hits = self.unit.skill_parameters.get(4).copied().unwrap_or(0.0);
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_scale
                * atk_scale;
            dps += skilldmg * skill_hits * (self.unit.targets as f64)
                / ((self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64))
                    + 1.2
                    + skill_hits);
            if self.unit.trait_damage {
                main_atk_buff = if self.unit.skill_damage {
                    2.0 + 0.8 * 6.0
                } else {
                    2.0
                };
                main_summon_atk = self.unit.drone_atk * (1.0 + self.unit.buff_atk + main_atk_buff)
                    + self.unit.buff_atk_flat;
                mainhit = ((main_summon_atk * (1.0 - res / 100.0)) as f64)
                    .max((main_summon_atk * 0.05) as f64)
                    * dmg_scale;
                maindps = mainhit / ((self.unit.drone_atk_interval as f64) + 0.7)
                    * self.unit.attack_speed
                    / 100.0;
                mainskilldps = ((main_summon_atk * 1.2 * (1.0 - res / 100.0)) as f64)
                    .max((main_summon_atk * 1.2 * 0.05) as f64)
                    * dmg_scale;
                dps += (maindps * 15.0 / (1.0 + (self.unit.sp_boost as f64))
                    + mainskilldps * 8.0 * (self.unit.targets as f64))
                    / (8.0 + 15.0 / (1.0 + (self.unit.sp_boost as f64)));
                if self.unit.talent_damage {
                    final_atk_summon = self.unit.drone_atk
                        * (1.0 + self.unit.buff_atk + summon_atk)
                        + self.unit.buff_atk_flat;
                    summondmg = ((final_atk_summon * (1.0 - res / 100.0)) as f64)
                        .max((final_atk_summon * 0.05) as f64)
                        * dmg_scale;
                    dps += 2.0 * summondmg / (self.unit.drone_atk_interval as f64)
                        * (self.unit.attack_speed)
                        / 100.0;
                }
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Necrass {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Necrass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
