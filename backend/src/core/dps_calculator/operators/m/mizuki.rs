//! DPS calculations for Mizuki
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Mizuki operator implementation
pub struct Mizuki {
    pub unit: OperatorUnit,
}

impl Mizuki {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Creates a new Mizuki operator
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
    /// bonusdmg = self.talent1_params[0] if self.elite > 0 else 0
    /// bonustargets = self.talent1_params[1] if self.elite > 0 else 0
    /// atkbuff = self.talent2_params[1] if self.talent2_dmg else 0
    /// aspd = 50 if self.module == 3 and self.module_dmg else 0
    /// if self.module == 3 and self.module_dmg and self.module_lvl > 1: bonustargets += 1
    /// if self.module == 3 and self.module_dmg and self.module_lvl == 3 and self.skill > 0: bonustargets += 1
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// talent_scale = self.skill_params[1] if self.skill == 1 else 1
    /// sp_cost = self.skill_cost/(1 + self.sp_boost) + 1.2 #sp lockout
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitbonus = np.fmax(final_atk * bonusdmg * (1-res/100), final_atk * bonusdmg * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// skillbonus = np.fmax(final_atk * bonusdmg * talent_scale * (1-res/100), final_atk * bonusdmg * talent_scale * 0.05)
    ///
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// avgarts = skillbonus
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[2] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// avgarts = (skillbonus + (atks_per_skillactivation -1) * hitbonus) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// avgarts = (skillbonus + int(atks_per_skillactivation) * hitbonus) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/(self.atk_interval/(self.attack_speed/100)) * self.targets + avgarts/(self.atk_interval/((self.attack_speed+aspd)/100)) * min(self.targets, bonustargets)
    ///
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[1]
    /// atk_interval = self.atk_interval + self.skill_params[0]
    /// bonustargets += 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * bonusdmg * (1-res/100), final_atk * bonusdmg * 0.05)
    /// dps = hitdmg/(atk_interval/(self.attack_speed/100)) * self.targets + hitdmgarts/(atk_interval/((self.attack_speed+aspd)/100)) * min(self.targets, bonustargets)
    ///
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[0]
    /// bonustargets += 2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * bonusdmg * (1-res/100), final_atk * bonusdmg * 0.05)
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) * self.targets + hitdmgarts/(self.atk_interval/((self.attack_speed+aspd)/100)) * min(self.targets, bonustargets)
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

        let mut avghit: f64 = 0.0;
        let mut avgarts: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut bonusdmg: f64 = 0.0;

        bonusdmg = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut bonustargets = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        atkbuff = if self.unit.talent2_damage {
            self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        aspd = if ((self.unit.module_index as f64) as f64) == 3.0 && self.unit.module_damage {
            50.0
        } else {
            0.0
        };
        if (self.unit.module_index as f64) == 3.0
            && self.unit.module_damage
            && (self.unit.module_level as f64) > 1.0
        {
            bonustargets += 1.0;
        }
        if (self.unit.module_index as f64) == 3.0
            && self.unit.module_damage
            && (self.unit.module_level as f64) == 3.0
            && (self.unit.skill_index as f64) > 0.0
        {
            bonustargets += 1.0;
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            let mut talent_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut hitbonus = ((final_atk * bonusdmg * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonusdmg * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            let mut skillbonus = ((final_atk * bonusdmg * talent_scale * (1.0 - res / 100.0))
                as f64)
                .max((final_atk * bonusdmg * talent_scale * 0.05) as f64);
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            avgarts = skillbonus;
            if atks_per_skillactivation > 1.0 {
                if self.unit.skill_parameters[2] > 1.0 {
                    avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                    avgarts = (skillbonus + (atks_per_skillactivation - 1.0) * hitbonus)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                    avgarts = (skillbonus + ((atks_per_skillactivation) as f64).trunc() * hitbonus)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * (self.unit.targets as f64)
                + avgarts
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * ((self.unit.targets as f64) as f64).min((bonustargets) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            atk_interval = (self.unit.attack_interval as f64)
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            bonustargets += 1.0;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * bonusdmg * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonusdmg * 0.05) as f64);
            dps = hitdmg / (atk_interval / (self.unit.attack_speed / 100.0))
                * (self.unit.targets as f64)
                + hitdmgarts / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0))
                    * ((self.unit.targets as f64) as f64).min((bonustargets) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            bonustargets += 2.0;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * bonusdmg * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonusdmg * 0.05) as f64);
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * (self.unit.targets as f64)
                + hitdmgarts
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * ((self.unit.targets as f64) as f64).min((bonustargets) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Mizuki {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Mizuki {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
