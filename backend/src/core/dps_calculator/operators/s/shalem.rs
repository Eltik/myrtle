//! DPS calculations for Shalem
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Shalem operator implementation
pub struct Shalem {
    pub unit: OperatorUnit,
}

impl Shalem {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Shalem operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// extra_scale = 0.1 if self.module == 1 else 0
    /// aspd = self.talent1_params[0] if self.talent_dmg else 0
    /// atkbuff = self.talent1_params[1] if self.talent_dmg else 0
    /// crate = self.talent2_params[0] if self.elite == 2 else 0
    /// newres = res * (1 + self.talent2_params[1]) if self.elite == 2 else res
    ///
    /// ####the actual skills
    /// if self.skill == 0:
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk -defense , final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed /100
    /// if self.skill == 1:
    /// atk_interval = self.atk_interval * (1 + self.skill_params[0])
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// countinghits = int( self.talent2_params[2] /(atk_interval/((self.attack_speed+aspd)/100))) + 1
    /// nocrit = (1-crate)**countinghits
    /// hitdmg = np.fmax(final_atk * (1+extra_scale) * (1-res/100), final_atk * (1+extra_scale) * 0.05)
    /// shreddmg = np.fmax(final_atk * (1+extra_scale) * (1-newres/100), final_atk * (1+extra_scale) * 0.05)
    /// avgdmg = hitdmg * nocrit + shreddmg * (1-nocrit)
    /// dps = avgdmg/atk_interval * (self.attack_speed+aspd)/100 * min(self.targets,3)
    /// if self.skill == 2:
    /// hits = self.skill_params[1]
    /// atk_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// countinghits =  (hits * int(self.talent2_params[2] /(self.atk_interval/((self.attack_speed+aspd)/100))) + 3)/self.targets + 1
    /// nocrit = (1-crate)**countinghits
    /// hitdmg = np.fmax(final_atk * (atk_scale + extra_scale) * (1-res/100), final_atk * (atk_scale + extra_scale) * 0.05)
    /// shreddmg = np.fmax(final_atk * (atk_scale + extra_scale) * (1-newres/100), final_atk * (atk_scale + extra_scale) * 0.05)
    /// avgdmg = hitdmg * nocrit + shreddmg * (1-nocrit)
    /// dps = 6 * avgdmg/self.atk_interval * (self.attack_speed+aspd)/100
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

        let mut shreddmg: f64 = 0.0;
        let mut nocrit: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut countinghits: f64 = 0.0;

        let mut extra_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        aspd = if self.unit.talent_damage {
            self.unit.talent1_parameters[0]
        } else {
            0.0
        };
        atkbuff = if self.unit.talent_damage {
            self.unit.talent1_parameters[1]
        } else {
            0.0
        };
        let mut crit_rate = if ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters[0]
        } else {
            0.0
        };
        newres = if ((self.unit.elite as f64) as f64) == 2.0 {
            res * (1.0 + self.unit.talent2_parameters[1])
        } else {
            res
        };
        // ###the actual skills
        if (self.unit.skill_index as f64) == 0.0 {
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            atk_interval =
                (self.unit.attack_interval as f64) * (1.0 + self.unit.skill_parameters[0]);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            countinghits = (self.unit.talent2_parameters[2]
                / (atk_interval / ((self.unit.attack_speed + aspd) as f64).trunc() / 100.0))
                + 1.0;
            nocrit = ((1.0 - crit_rate) as f64).powf(countinghits as f64);
            hitdmg = ((final_atk * (1.0 + extra_scale) * (1.0 - res / 100.0)) as f64)
                .max((final_atk * (1.0 + extra_scale) * 0.05) as f64);
            shreddmg = ((final_atk * (1.0 + extra_scale) * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * (1.0 + extra_scale) * 0.05) as f64);
            avgdmg = hitdmg * nocrit + shreddmg * (1.0 - nocrit);
            dps = avgdmg / atk_interval * (self.unit.attack_speed + aspd) / 100.0
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut hits = self.unit.skill_parameters[1];
            atk_scale = self.unit.skill_parameters[0];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            countinghits = (hits
                * (self.unit.talent2_parameters[2]
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) as f64).trunc()
                        / 100.0))
                + 3.0)
                / (self.unit.targets as f64)
                + 1.0;
            nocrit = ((1.0 - crit_rate) as f64).powf(countinghits as f64);
            hitdmg = ((final_atk * (atk_scale + extra_scale) * (1.0 - res / 100.0)) as f64)
                .max((final_atk * (atk_scale + extra_scale) * 0.05) as f64);
            shreddmg = ((final_atk * (atk_scale + extra_scale) * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * (atk_scale + extra_scale) * 0.05) as f64);
            avgdmg = hitdmg * nocrit + shreddmg * (1.0 - nocrit);
            dps = 6.0 * avgdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        dps
    }
}

impl std::ops::Deref for Shalem {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Shalem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
