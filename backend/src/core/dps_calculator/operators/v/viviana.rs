//! DPS calculations for Viviana
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Viviana operator implementation
pub struct Viviana {
    pub unit: OperatorUnit,
}

impl Viviana {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[3];

    /// Creates a new Viviana operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            3, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// dmg_scale = 1 + self.talent1_params[1] * 2 if self.talent_dmg else 1 + self.talent1_params[1]
    /// if self.elite == 0: dmg_scale = 1
    /// burn_res = np.fmax(0,res-20)
    /// fallout_dmg = 7000
    /// ele_scale = 0.15
    /// ele_appli = self.talent1_params[0] if self.module == 3 and self.module_lvl > 1 else 0
    /// if self.talent_dmg: ele_appli *= 2
    /// ele_gauge = 1000
    /// if not self.talent2_dmg: ele_gauge = 2000
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// sp_cost = self.skill_cost/(1 + self.sp_boost) + 1.2 #sp lockout
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg_scale
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_scale * 2
    /// hitdmgarts2 = np.fmax(final_atk * (1-burn_res/100), final_atk * 0.05) * dmg_scale
    /// skilldmg2 = np.fmax(final_atk * skill_scale * (1-burn_res/100), final_atk * skill_scale * 0.05) * dmg_scale * 2
    /// if self.skill == 0:
    /// skilldmg = hitdmgarts
    /// skilldmg2 = hitdmgarts2
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// avghit2 = skilldmg2
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmgarts) / (int(atks_per_skillactivation)+1)
    /// avghit2 = (skilldmg2 + int(atks_per_skillactivation) * hitdmgarts2) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    /// if self.module == 3 and self.module_dmg and self.module_lvl > 1:
    /// time_to_trigger = ele_gauge / (dps*ele_appli)
    /// fallout_dps = (avghit2 + ele_scale * final_atk)/self.atk_interval * self.attack_speed/100
    /// dps = (dps * time_to_trigger + fallout_dps * 10 + fallout_dmg) / (time_to_trigger + 10)
    ///
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// aspd = self.skill_params[6] if self.skill_dmg else 0
    /// crate = 0.2
    /// cdmg = self.skill_params[3]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg_scale
    /// skilldmg = 2 * np.fmax(final_atk * cdmg * (1-res/100), final_atk * cdmg * 0.05) * dmg_scale
    /// avgdmg = crate * skilldmg + (1-crate) * hitdmgarts
    /// hitdmgarts2 = np.fmax(final_atk * (1-burn_res/100), final_atk * 0.05) * dmg_scale
    /// skilldmg2 = 2 * np.fmax(final_atk * cdmg * (1-burn_res/100), final_atk * cdmg * 0.05) * dmg_scale
    /// avgdmg2 = crate * skilldmg2 + (1-crate) * hitdmgarts2
    /// dps = avgdmg/self.atk_interval * (self.attack_speed+aspd)/100 * min(self.targets,2)
    /// if self.module == 3 and self.module_dmg and self.module_lvl > 1:
    /// time_to_trigger = ele_gauge / (dps*ele_appli/min(self.targets,2))
    /// fallout_dps = (avgdmg2 + ele_scale * final_atk)/self.atk_interval * (self.attack_speed+aspd)/100
    /// dps = (dps * time_to_trigger + fallout_dps * 10 + fallout_dmg) / (time_to_trigger + 10)
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[1]
    /// hits = 3 if self.skill_dmg else 2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg_scale
    /// hitdmgarts2 = np.fmax(final_atk * (1-burn_res/100), final_atk * 0.05) * dmg_scale
    /// dps = hits * hitdmgarts/1.75 * self.attack_speed/100
    /// if self.module == 3 and self.module_dmg and self.module_lvl > 1:
    /// time_to_trigger = ele_gauge / (dps*ele_appli)
    /// fallout_dps = hits * (hitdmgarts2 + ele_scale * final_atk)/1.75 * self.attack_speed/100
    /// dps = (dps * time_to_trigger + fallout_dps * 10 + fallout_dmg) / (time_to_trigger + 10)
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

        let mut avghit2: f64 = 0.0;
        let mut fallout_dps: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut hitdmgarts2: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut skilldmg2: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut time_to_trigger: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut ele_scale: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut atks_per_skillactivation: f64 = 0.0;
        let mut atkcycle: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut aspd: f64 = 0.0;

        let mut dmg_scale = if self.unit.talent_damage {
            1.0 + self.unit.talent1_parameters[1] * 2.0
        } else {
            1.0 + self.unit.talent1_parameters[1]
        };
        if (self.unit.elite as f64) == 0.0 {
            dmg_scale = 1.0;
        }
        let mut burn_res = ((0) as f64).max((res - 20.0) as f64);
        let mut fallout_dmg = 7000.0;
        ele_scale = 0.15;
        let mut ele_appli = if ((self.unit.module_index as f64) as f64) == 3.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            self.unit.talent1_parameters[0]
        } else {
            0.0
        };
        // UNTRANSLATED: if self.talent_dmg: ele_appli *= 2
        let mut ele_gauge = 1000.0;
        if !self.unit.talent2_damage {
            ele_gauge = 2000.0;
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters[0];
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale;
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_scale
                * 2.0;
            hitdmgarts2 = ((final_atk * (1.0 - burn_res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * dmg_scale;
            skilldmg2 = ((final_atk * skill_scale * (1.0 - burn_res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_scale
                * 2.0;
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmg = hitdmgarts;
                skilldmg2 = hitdmgarts2;
            }
            atkcycle = (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            avghit2 = skilldmg2;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmgarts)
                    / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                avghit2 = (skilldmg2 + ((atks_per_skillactivation) as f64).trunc() * hitdmgarts2)
                    / (((atks_per_skillactivation) as f64).trunc() + 1.0);
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if (self.unit.module_index as f64) == 3.0
                && self.unit.module_damage
                && (self.unit.module_level as f64) > 1.0
            {
                time_to_trigger = ele_gauge / (dps * ele_appli);
                fallout_dps = (avghit2 + ele_scale * final_atk)
                    / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0;
                dps = (dps * time_to_trigger + fallout_dps * 10.0 + fallout_dmg)
                    / (time_to_trigger + 10.0);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters[0];
            aspd = if self.unit.skill_damage {
                self.unit.skill_parameters[6]
            } else {
                0.0
            };
            let mut crit_rate = 0.2;
            cdmg = self.unit.skill_parameters[3];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale;
            skilldmg = 2.0
                * ((final_atk * cdmg * (1.0 - res / 100.0)) as f64)
                    .max((final_atk * cdmg * 0.05) as f64)
                * dmg_scale;
            avgdmg = crit_rate * skilldmg + (1.0 - crit_rate) * hitdmgarts;
            hitdmgarts2 = ((final_atk * (1.0 - burn_res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * dmg_scale;
            skilldmg2 = 2.0
                * ((final_atk * cdmg * (1.0 - burn_res / 100.0)) as f64)
                    .max((final_atk * cdmg * 0.05) as f64)
                * dmg_scale;
            let mut avgdmg2 = crit_rate * skilldmg2 + (1.0 - crit_rate) * hitdmgarts2;
            dps = avgdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
            if (self.unit.module_index as f64) == 3.0
                && self.unit.module_damage
                && (self.unit.module_level as f64) > 1.0
            {
                time_to_trigger = ele_gauge
                    / (dps * ele_appli / ((self.unit.targets as f64) as f64).min((2) as f64));
                fallout_dps = (avgdmg2 + ele_scale * final_atk)
                    / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed + aspd)
                    / 100.0;
                dps = (dps * time_to_trigger + fallout_dps * 10.0 + fallout_dmg)
                    / (time_to_trigger + 10.0);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters[1];
            let mut hits = if self.unit.skill_damage { 3.0 } else { 2.0 };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale;
            hitdmgarts2 = ((final_atk * (1.0 - burn_res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * dmg_scale;
            dps = hits * hitdmgarts / 1.75 * self.unit.attack_speed / 100.0;
            if (self.unit.module_index as f64) == 3.0
                && self.unit.module_damage
                && (self.unit.module_level as f64) > 1.0
            {
                time_to_trigger = ele_gauge / (dps * ele_appli);
                fallout_dps = hits * (hitdmgarts2 + ele_scale * final_atk) / 1.75
                    * self.unit.attack_speed
                    / 100.0;
                dps = (dps * time_to_trigger + fallout_dps * 10.0 + fallout_dmg)
                    / (time_to_trigger + 10.0);
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Viviana {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Viviana {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
