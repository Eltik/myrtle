//! DPS calculations for Ifrit
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Ifrit operator implementation
pub struct Ifrit {
    pub unit: OperatorUnit,
}

impl Ifrit {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 3];

    /// Creates a new Ifrit operator
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
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// resshred = self.talent1_params[0]
    /// ele_gauge = 1000 if self.module_dmg else 2000
    /// burnres = np.fmax(0,res-20)
    ///
    /// recovery_interval = self.talent2_params[1]
    /// sp_recovered = self.talent2_params[0] if self.elite == 2 else 0
    /// if self.module == 1:
    /// if self.module_lvl == 2: sp_recovered = 3
    /// if self.module_lvl == 3: sp_recovered = 2 + 0.3 * 5
    ///
    /// ####the actual skills
    /// if self.skill < 2:
    /// atkbuff = self.skill_params[1] if self.skill == 1 else 0
    /// aspd = self.skill_params[0] if self.skill == 1 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// newres = res * (1+resshred)
    /// hitdmgarts = np.fmax(final_atk *atk_scale *(1-newres/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// if self.module == 3 and self.talent_dmg and self.module_lvl > 1:
    /// time_to_proc = ele_gauge * self.targets / (dps*0.08)
    /// newres2 = burnres * (1 + resshred)
    /// hitdmgarts = np.fmax(final_atk *(1-newres2/100), final_atk * 0.05)
    /// ele_hit = final_atk * (0.2*0.1*self.module_lvl)/(1+self.buff_fragile) if self.module_lvl > 1 else 0
    /// fallout_dps = (hitdmgarts + ele_hit)/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// dps = (dps * time_to_proc + 10 * fallout_dps + 7000/(1+self.buff_fragile))/(time_to_proc+10)
    ///
    /// if self.skill == 2:
    /// sp_cost = self.skill_cost
    /// skill_scale = self.skill_params[0]
    /// burn_scale = 0.99
    /// newres = res * (1+resshred)
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * atk_scale *(1-newres/100), final_atk * atk_scale * 0.05)
    /// skilldmgarts = np.fmax(final_atk * atk_scale * skill_scale * (1-newres/100), final_atk * atk_scale * skill_scale * 0.05)
    /// burndmg = np.fmax(final_atk * burn_scale *(1-newres/100), final_atk * burn_scale * 0.05)
    /// sp_cost = sp_cost / (1+sp_recovered/recovery_interval + self.sp_boost) + 1.2 #talent bonus recovery + sp lockout
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmgarts + burndmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmgarts + burndmg + (atks_per_skillactivation - 1) * hitdmgarts) / atks_per_skillactivation
    /// dps = avghit/self.atk_interval * self.attack_speed/100 * self.targets
    ///
    /// if self.module == 3 and self.talent_dmg and self.module_lvl > 1:
    /// time_to_proc = ele_gauge * self.targets / (dps*0.08)
    /// newres2 = burnres * (1 + resshred)
    /// hitdmgarts = np.fmax(final_atk *(1-newres2/100), final_atk * 0.05)
    /// skilldmgarts = np.fmax(final_atk * skill_scale *(1-newres2/100), final_atk * skill_scale * 0.05)
    /// burndmg = np.fmax(final_atk * burn_scale * (1-newres2/100), final_atk * burn_scale * 0.05)
    /// ele_hit = final_atk * (0.2*0.1*self.module_lvl)/(1+self.buff_fragile) if self.module_lvl > 1 else 0
    /// avghit = skilldmgarts + burndmg + ele_hit
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmgarts + burndmg + (atks_per_skillactivation - 1) * hitdmgarts + ele_hit) / atks_per_skillactivation
    /// fallout_dps = (avghit + ele_hit)/self.atk_interval * self.attack_speed/100 * self.targets
    /// dps = (dps * time_to_proc + 10 * fallout_dps + 7000/(1+self.buff_fragile))/(time_to_proc+10)
    ///
    /// if self.skill == 3:
    /// atk_scale *= self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// flatshred = -self.skill_params[2]
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// res = res / self.shreds[0]
    /// newres = np.fmax(0, res-flatshred)
    /// newres = newres * (1+resshred)
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// newres *= self.shreds[2]
    /// hitdmgarts = np.fmax(final_atk *atk_scale *(1-newres/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmgarts * self.targets
    ///
    /// if self.module == 3 and self.talent_dmg and self.module_lvl > 1:
    /// time_to_proc = ele_gauge * self.targets / (dps*0.08)
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// res = res / self.shreds[0]
    /// newres2 = np.fmax(0, res-flatshred-20)
    /// newres2 = newres2 * (1+resshred)
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// newres2 *= self.shreds[2]
    /// hitdmgarts = np.fmax(final_atk *atk_scale *(1-newres2/100), final_atk * atk_scale * 0.05)
    /// ele_hit = final_atk * (0.2*0.1*self.module_lvl)/(1+self.buff_fragile) if self.module_lvl > 1 else 0
    /// fallout_dps = (hitdmgarts + ele_hit) * self.targets
    /// dps = (dps * time_to_proc + 10 * fallout_dps + 7000/(1+self.buff_fragile))/(time_to_proc+10)
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut burndmg: f64 = 0.0;
        let mut ele_hit: f64 = 0.0;
        let mut newres2: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmgarts: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut res: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut fallout_dps: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut time_to_proc: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        let mut resshred = self.unit.talent1_parameters[0];
        let mut ele_gauge = if self.unit.module_damage {
            1000.0
        } else {
            2000.0
        };
        let mut burnres = ((0) as f64).max((res - 20.0) as f64);
        let mut recovery_interval = self.unit.talent2_parameters[1];
        let mut sp_recovered = if ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters[0]
        } else {
            0.0
        };
        if (self.unit.module_index as f64) == 1.0 {
            if (self.unit.module_level as f64) == 2.0 {
                sp_recovered = 3.0;
            }
            if (self.unit.module_level as f64) == 3.0 {
                sp_recovered = 2.0 + 0.3 * 5.0;
            }
            // ###the actual skills
        }
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters[1]
            } else {
                0.0
            };
            aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters[0]
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            newres = res * (1.0 + resshred);
            hitdmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmgarts / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
            if (self.unit.module_index as f64) == 3.0
                && self.unit.talent_damage
                && (self.unit.module_level as f64) > 1.0
            {
                time_to_proc = ele_gauge * (self.unit.targets as f64) / (dps * 0.08);
                newres2 = burnres * (1.0 + resshred);
                hitdmgarts =
                    ((final_atk * (1.0 - newres2 / 100.0)) as f64).max((final_atk * 0.05) as f64);
                ele_hit = if ((self.unit.module_level as f64) as f64) > 1.0 {
                    final_atk * (0.2 * 0.1 * ((self.unit.module_level as f64) as f64))
                        / (1.0 + self.unit.buff_fragile)
                } else {
                    0.0
                };
                fallout_dps = (hitdmgarts + ele_hit) / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed + aspd)
                    / 100.0
                    * (self.unit.targets as f64);
                dps = (dps * time_to_proc
                    + 10.0 * fallout_dps
                    + 7000.0 / (1.0 + self.unit.buff_fragile))
                    / (time_to_proc + 10.0);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            sp_cost = (self.unit.skill_cost as f64);
            skill_scale = self.unit.skill_parameters[0];
            let mut burn_scale = 0.99;
            newres = res * (1.0 + resshred);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            skilldmgarts = ((final_atk * atk_scale * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            burndmg = ((final_atk * burn_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * burn_scale * 0.05) as f64);
            sp_cost = sp_cost
                / (1.0 + sp_recovered / recovery_interval + (self.unit.sp_boost as f64))
                + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmgarts + burndmg;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmgarts + burndmg + (atks_per_skillactivation - 1.0) * hitdmgarts)
                    / atks_per_skillactivation;
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
            if (self.unit.module_index as f64) == 3.0
                && self.unit.talent_damage
                && (self.unit.module_level as f64) > 1.0
            {
                time_to_proc = ele_gauge * (self.unit.targets as f64) / (dps * 0.08);
                newres2 = burnres * (1.0 + resshred);
                hitdmgarts =
                    ((final_atk * (1.0 - newres2 / 100.0)) as f64).max((final_atk * 0.05) as f64);
                skilldmgarts = ((final_atk * skill_scale * (1.0 - newres2 / 100.0)) as f64)
                    .max((final_atk * skill_scale * 0.05) as f64);
                burndmg = ((final_atk * burn_scale * (1.0 - newres2 / 100.0)) as f64)
                    .max((final_atk * burn_scale * 0.05) as f64);
                ele_hit = if ((self.unit.module_level as f64) as f64) > 1.0 {
                    final_atk * (0.2 * 0.1 * ((self.unit.module_level as f64) as f64))
                        / (1.0 + self.unit.buff_fragile)
                } else {
                    0.0
                };
                avghit = skilldmgarts + burndmg + ele_hit;
                if atks_per_skillactivation > 1.0 {
                    avghit = (skilldmgarts
                        + burndmg
                        + (atks_per_skillactivation - 1.0) * hitdmgarts
                        + ele_hit)
                        / atks_per_skillactivation;
                }
                fallout_dps = (avghit + ele_hit) / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0
                    * (self.unit.targets as f64);
                dps = (dps * time_to_proc
                    + 10.0 * fallout_dps
                    + 7000.0 / (1.0 + self.unit.buff_fragile))
                    / (time_to_proc + 10.0);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_scale *= self.unit.skill_parameters[0];
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut flatshred = -self.unit.skill_parameters[2];
            if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
                res /= self.unit.shreds[0];
            }
            newres = ((0) as f64).max((res - flatshred) as f64);
            newres *= (1.0 + resshred);
            if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
                newres *= self.unit.shreds[2];
            }
            hitdmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmgarts * (self.unit.targets as f64);
            if (self.unit.module_index as f64) == 3.0
                && self.unit.talent_damage
                && (self.unit.module_level as f64) > 1.0
            {
                time_to_proc = ele_gauge * (self.unit.targets as f64) / (dps * 0.08);
                if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
                    res /= self.unit.shreds[0];
                }
                newres2 = ((0) as f64).max((res - flatshred - 20.0) as f64);
                newres2 *= (1.0 + resshred);
                if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
                    newres2 *= self.unit.shreds[2];
                }
                hitdmgarts = ((final_atk * atk_scale * (1.0 - newres2 / 100.0)) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64);
                ele_hit = if ((self.unit.module_level as f64) as f64) > 1.0 {
                    final_atk * (0.2 * 0.1 * ((self.unit.module_level as f64) as f64))
                        / (1.0 + self.unit.buff_fragile)
                } else {
                    0.0
                };
                fallout_dps = (hitdmgarts + ele_hit) * (self.unit.targets as f64);
                dps = (dps * time_to_proc
                    + 10.0 * fallout_dps
                    + 7000.0 / (1.0 + self.unit.buff_fragile))
                    / (time_to_proc + 10.0);
            }
        }
        dps
    }
}

impl std::ops::Deref for Ifrit {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ifrit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
