//! DPS calculations for TogawaSakiko
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// TogawaSakiko operator implementation
pub struct TogawaSakiko {
    pub unit: OperatorUnit,
}

impl TogawaSakiko {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new TogawaSakiko operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
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
    /// atk_scale = 1 #if self.trait_dmg else 0.8
    /// atkbuff = self.skill_params[1] if self.skill == 2 and not self.skill_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// aspd = self.talent2_params[1]
    /// aspd += 12 if self.module == 2 and self.module_dmg else 0
    /// resshred = 0
    /// defshred = 0
    /// if self.talent_dmg:
    /// resshred = self.talent1_params[4] * self.talent1_params[1] if self.module == 2 and self.module_lvl > 1 else self.talent1_params[4] * self.talent1_params[2]
    /// defshred = self.talent1_params[4] * self.talent1_params[0] if self.module == 2 and self.module_lvl > 1 else self.talent1_params[4] * self.talent1_params[1]
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// res = res / self.shreds[0]
    /// newres = res * (1-resshred)
    /// if self.shreds[0] < 1 and self.shreds[0] > 0:
    /// defense = defense / self.shreds[0]
    /// newdef = defense * (1-defshred)
    ///
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0] if self.skill_dmg else self.skill_params[7]
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// skill_dmg = np.fmax(final_atk * skill_scale * (1-newres/100), final_atk * skill_scale * 0.05) * 8
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100
    /// dps += skill_dmg / self.atk_interval * (self.attack_speed + aspd) / 100 / self.skill_cost
    /// if self.talent2_dmg:
    /// dps = skill_dmg / self.atk_interval * (self.attack_speed + aspd) / 100
    ///
    /// if self.skill == 2:
    /// hits = 2 if self.talent2_dmg else 1
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05) * hits
    /// hitdmgarts = np.fmax(final_atk * atk_scale * (1-newres/100), final_atk * atk_scale * 0.05) * hits
    /// if self.skill_dmg:
    /// dps = hitdmgarts / self.atk_interval * (self.attack_speed + aspd + self.skill_params[0]) / 100
    /// else:
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * self.targets
    ///
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * atk_scale * skill_scale * 0.05)
    /// hitdmgarts = np.fmax(final_atk * atk_scale * skill_scale * (1-newres/100), final_atk * atk_scale * skill_scale * 0.05)
    /// dps = 2*(hitdmg+hitdmgarts) / self.atk_interval * (self.attack_speed + aspd) / 100
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
        let defense = enemy.defense;
        let res = enemy.res;

        let mut atkbuff: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        atk_scale = 1.0;
        atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 && !self.unit.skill_damage {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        aspd = self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0);
        aspd += if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            12.0
        } else {
            0.0
        };
        let mut resshred = 0.0;
        defshred = 0.0;
        if self.unit.talent_damage {
            resshred = if ((self.unit.module_index as f64) as f64) == 2.0
                && ((self.unit.module_level as f64) as f64) > 1.0
            {
                self.unit.talent1_parameters.get(4).copied().unwrap_or(0.0)
                    * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                self.unit.talent1_parameters.get(4).copied().unwrap_or(0.0)
                    * self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
            };
            defshred = if ((self.unit.module_index as f64) as f64) == 2.0
                && ((self.unit.module_level as f64) as f64) > 1.0
            {
                self.unit.talent1_parameters.get(4).copied().unwrap_or(0.0)
                    * self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            } else {
                self.unit.talent1_parameters.get(4).copied().unwrap_or(0.0)
                    * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
            };
        }
        if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
            let mut res = res / self.unit.shreds.first().copied().unwrap_or(0.0);
        }
        newres = res * (1.0 - resshred);
        if self.unit.shreds[0] < 1.0 && self.unit.shreds[0] > 0.0 {
            let mut defense = defense / self.unit.shreds.first().copied().unwrap_or(0.0);
        }
        let mut newdef = defense * (1.0 - defshred);
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = if self.unit.skill_damage {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                self.unit.skill_parameters.get(7).copied().unwrap_or(0.0)
            };
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut skill_dmg = ((final_atk * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * 8.0;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
            dps += skill_dmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                / (self.unit.skill_cost as f64);
            if self.unit.talent2_damage {
                dps = skill_dmg / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed + aspd)
                    / 100.0;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut hits = if self.unit.talent2_damage { 2.0 } else { 1.0 };
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * hits;
            hitdmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * hits;
            if self.unit.skill_damage {
                dps = hitdmgarts / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed
                        + aspd
                        + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                    / 100.0;
            } else {
                dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                    / 100.0
                    * (self.unit.targets as f64);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            hitdmgarts = ((final_atk * atk_scale * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = 2.0 * (hitdmg + hitdmgarts) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for TogawaSakiko {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for TogawaSakiko {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
