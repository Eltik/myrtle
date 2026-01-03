//! DPS calculations for Logos
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Logos operator implementation
pub struct Logos {
    pub unit: OperatorUnit,
}

impl Logos {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[3, 2];

    /// Creates a new Logos operator
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
    /// bonuschance = self.talent1_params[0] if self.elite > 0 else 0
    /// if self.module == 3: bonuschance += 0.1 * (self.module_lvl - 1)
    /// bonusdmg = self.talent1_params[1]
    /// bonus_hitcount = 2 if self.module == 2 and self.module_lvl > 1 else 1
    /// falloutdmg = 0.2 * self.module_lvl /(1+self.buff_fragile) if self.module == 3 and self.module_lvl > 1 else 0
    /// newres = np.fmax(0,res-10) if self.elite == 2 else res
    /// if self.elite == 2:
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// res = res / self.shreds[2]
    /// newres = np.fmax(0, res    - 10)
    /// if self.shreds[2] < 1 and self.shreds[2] > 0:
    /// newres *= self.shreds[2]
    /// shreddmg = self.talent2_params[2] if self.elite == 2 else 0
    ///
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]*self.skill) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-newres/100), final_atk * 0.05) + np.fmax(shreddmg * (1-newres/100), shreddmg * 0.05)
    /// bonusdmg = (np.fmax(final_atk * bonusdmg * (1-newres/100), final_atk * bonusdmg * 0.05) + np.fmax(shreddmg * (1-newres/100), shreddmg * 0.05)) * bonuschance * bonus_hitcount
    /// dps = (hitdmg+bonusdmg)/self.atk_interval * self.attack_speed/100
    /// if self.module == 3 and self.talent_dmg:
    /// ele_gauge = 1000 if self.module_dmg else 2000
    /// eledps = dps * 0.08
    /// fallouttime = ele_gauge / eledps
    /// dps += 12000/(fallouttime + 15)/(1+self.buff_fragile)
    /// if self.module_lvl > 1:
    /// dps += final_atk * falloutdmg /self.atk_interval * self.attack_speed/100 * bonuschance * 15 / (fallouttime + 15)
    ///
    /// if self.skill == 2:
    /// scaling = self.skill_params[2]
    /// if self.skill_dmg: scaling *= 3
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * scaling * (1-newres/100), final_atk * scaling * 0.05) + np.fmax(shreddmg * (1-newres/100), shreddmg * 0.05)
    /// bonusdmg = (np.fmax(final_atk * bonusdmg * (1-newres/100), final_atk * bonusdmg * 0.05) + np.fmax(shreddmg * (1-newres/100), shreddmg * 0.05)) * bonuschance * bonus_hitcount
    /// dps = (hitdmg+bonusdmg) * 2
    /// if self.module == 3 and self.talent_dmg:
    /// ele_gauge = 1000 if self.module_dmg else 2000
    /// eledps = dps * 0.08
    /// fallouttime = ele_gauge / eledps
    /// dps += 12000/(fallouttime + 15)/(1+self.buff_fragile)
    /// if self.module_lvl > 1:
    /// dps += final_atk * falloutdmg * 2 * bonuschance * 15 / (fallouttime + 15)
    ///
    /// if self.skill == 3:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-newres/100), final_atk * 0.05) + np.fmax(shreddmg * (1-newres/100), shreddmg * 0.05)
    /// bonusdmg = (np.fmax(final_atk * bonusdmg * (1-newres/100), final_atk * bonusdmg * 0.05) + np.fmax(shreddmg * (1-newres/100), shreddmg * 0.05)) * bonuschance * bonus_hitcount
    /// dps = (hitdmg+bonusdmg)/self.atk_interval * self.attack_speed/100 * min(self.targets,self.skill_params[1])
    /// if self.module == 3 and self.talent_dmg:
    /// ele_gauge = 1000 if self.module_dmg else 2000
    /// eledps = dps * 0.08 / min(self.targets,self.skill_params[1])
    /// fallouttime = ele_gauge / eledps
    /// dps += 12000/(fallouttime + 15) * min(self.targets,self.skill_params[1]) /(1+self.buff_fragile)
    /// if self.module_lvl > 1:
    /// dps += final_atk * falloutdmg/self.atk_interval * self.attack_speed/100 * min(self.targets,self.skill_params[1]) * bonuschance * 15 / (fallouttime + 15)
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

        let mut bonusdmg: f64 = 0.0;
        let mut eledps: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut res: f64 = 0.0;
        let mut ele_gauge: f64 = 0.0;
        let mut fallouttime: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        let mut bonuschance = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters[0]
        } else {
            0.0
        };
        // UNTRANSLATED: if self.module == 3: bonuschance += 0.1 * (self.module_lvl - 1)
        bonusdmg = self.unit.talent1_parameters[1];
        let mut bonus_hitcount = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            2.0
        } else {
            1.0
        };
        let mut falloutdmg = if ((self.unit.module_index as f64) as f64) == 3.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            0.2 * ((self.unit.module_level as f64) as f64) / (1.0 + self.unit.buff_fragile)
        } else {
            0.0
        };
        newres = if ((self.unit.elite as f64) as f64) == 2.0 {
            ((0) as f64).max((res - 10.0) as f64)
        } else {
            res
        };
        if (self.unit.elite as f64) == 2.0 {
            if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
                res = res / self.unit.shreds[2];
            }
            newres = ((0) as f64).max((res - 10.0) as f64);
            if self.unit.shreds[2] < 1.0 && self.unit.shreds[2] > 0.0 {
                newres *= self.unit.shreds[2];
            }
        }
        let mut shreddmg = if ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters[2]
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters[0] * (self.unit.skill_index as f64))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64)
                + ((shreddmg * (1.0 - newres / 100.0)) as f64).max((shreddmg * 0.05) as f64);
            bonusdmg = (((final_atk * bonusdmg * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * bonusdmg * 0.05) as f64)
                + ((shreddmg * (1.0 - newres / 100.0)) as f64).max((shreddmg * 0.05) as f64))
                * bonuschance
                * bonus_hitcount;
            dps = (hitdmg + bonusdmg) / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0;
            if (self.unit.module_index as f64) == 3.0 && self.unit.talent_damage {
                ele_gauge = if self.unit.module_damage {
                    1000.0
                } else {
                    2000.0
                };
                eledps = dps * 0.08;
                fallouttime = ele_gauge / eledps;
                dps += 12000.0 / (fallouttime + 15.0) / (1.0 + self.unit.buff_fragile);
                if (self.unit.module_level as f64) > 1.0 {
                    dps += final_atk * falloutdmg / (self.unit.attack_interval as f64)
                        * self.unit.attack_speed
                        / 100.0
                        * bonuschance
                        * 15.0
                        / (fallouttime + 15.0);
                }
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut scaling = self.unit.skill_parameters[2];
            // UNTRANSLATED: if self.skill_dmg: scaling *= 3
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * scaling * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * scaling * 0.05) as f64)
                + ((shreddmg * (1.0 - newres / 100.0)) as f64).max((shreddmg * 0.05) as f64);
            bonusdmg = (((final_atk * bonusdmg * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * bonusdmg * 0.05) as f64)
                + ((shreddmg * (1.0 - newres / 100.0)) as f64).max((shreddmg * 0.05) as f64))
                * bonuschance
                * bonus_hitcount;
            dps = (hitdmg + bonusdmg) * 2.0;
            if (self.unit.module_index as f64) == 3.0 && self.unit.talent_damage {
                ele_gauge = if self.unit.module_damage {
                    1000.0
                } else {
                    2000.0
                };
                eledps = dps * 0.08;
                fallouttime = ele_gauge / eledps;
                dps += 12000.0 / (fallouttime + 15.0) / (1.0 + self.unit.buff_fragile);
                if (self.unit.module_level as f64) > 1.0 {
                    dps += final_atk * falloutdmg * 2.0 * bonuschance * 15.0 / (fallouttime + 15.0);
                }
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[0])
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64)
                + ((shreddmg * (1.0 - newres / 100.0)) as f64).max((shreddmg * 0.05) as f64);
            bonusdmg = (((final_atk * bonusdmg * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * bonusdmg * 0.05) as f64)
                + ((shreddmg * (1.0 - newres / 100.0)) as f64).max((shreddmg * 0.05) as f64))
                * bonuschance
                * bonus_hitcount;
            dps = (hitdmg + bonusdmg) / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                * ((self.unit.targets as f64) as f64).min((self.unit.skill_parameters[1]) as f64);
            if (self.unit.module_index as f64) == 3.0 && self.unit.talent_damage {
                ele_gauge = if self.unit.module_damage {
                    1000.0
                } else {
                    2000.0
                };
                eledps = dps * 0.08
                    / ((self.unit.targets as f64) as f64)
                        .min((self.unit.skill_parameters[1]) as f64);
                fallouttime = ele_gauge / eledps;
                dps += 12000.0 / (fallouttime + 15.0)
                    * ((self.unit.targets as f64) as f64)
                        .min((self.unit.skill_parameters[1]) as f64)
                    / (1.0 + self.unit.buff_fragile);
                if (self.unit.module_level as f64) > 1.0 {
                    dps += final_atk * falloutdmg / (self.unit.attack_interval as f64)
                        * self.unit.attack_speed
                        / 100.0
                        * ((self.unit.targets as f64) as f64)
                            .min((self.unit.skill_parameters[1]) as f64)
                        * bonuschance
                        * 15.0
                        / (fallouttime + 15.0);
                }
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Logos {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Logos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
