//! DPS calculations for Ebenholz
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Ebenholz operator implementation
pub struct Ebenholz {
    pub unit: OperatorUnit,
}

impl Ebenholz {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Creates a new Ebenholz operator
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
    /// aspd = 30 if self.module_dmg and self.module == 2 else 0
    /// atk_scale = self.talent1_params[0] if self.talent_dmg and self.elite > 0 else 1
    /// eledmg = 0
    /// bonus_scale = self.talent2_params[0] if self.targets == 1 and self.elite == 2 else 0
    /// eledmg = self.module_lvl * 0.1 /(1+self.buff_fragile) if self.module == 3 and self.module_lvl > 1 and self.talent2_dmg else 0
    /// extra_scale = self.talent2_params[3] if self.module == 2 and self.module_lvl > 1 else 0
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[1] if self.skill == 1 else 1
    /// atk_interval = self.atk_interval * self.skill_params[0] if self.skill == 1 else self.atk_interval
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale * (1-res/100), final_atk * atk_scale * skill_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus_scale * (1-res/100), final_atk * bonus_scale * 0.05)
    /// extradmg = np.fmax(final_atk * extra_scale * (1-res/100), final_atk * extra_scale * 0.05)
    /// dps = hitdmg/(atk_interval/((self.attack_speed + aspd)/100))
    /// if self.module == 3:
    /// ele_gauge = 1000 if self.module_dmg else 2000
    /// eledps = dps * 0.08
    /// fallouttime = ele_gauge / eledps
    /// dps += 12000/(fallouttime + 15)/(1+self.buff_fragile)
    /// dps += eledmg * final_atk /(self.atk_interval/((self.attack_speed + aspd)/100)) * 15/(fallouttime + 15)
    /// if self.targets == 1:
    /// dps += bonusdmg/(atk_interval/((self.attack_speed + aspd)/100))
    /// if self.targets > 1 and self.module == 2:
    /// dps += extradmg/(atk_interval/((self.attack_speed + aspd)/100)) * (self.targets -1)
    ///
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[1]
    /// if self.talent_dmg:
    /// atk_scale *= self.skill_params[2]
    /// aspd += self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus_scale * (1-res/100), final_atk * bonus_scale * 0.05)
    /// extradmg = np.fmax(final_atk * extra_scale * (1-res/100), final_atk * extra_scale * 0.05)
    ///
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed + aspd)/100))
    /// if self.module == 3:
    /// ele_gauge = 1000 if self.module_dmg else 2000
    /// eledps = dps * 0.08
    /// fallouttime = ele_gauge / eledps
    /// dps += 12000/(fallouttime + 15)/(1+self.buff_fragile)
    /// dps += eledmg * final_atk /(self.atk_interval/((self.attack_speed + aspd)/100)) * 15/(fallouttime + 15)
    /// if self.targets == 1:
    /// dps += bonusdmg/(self.atk_interval/((self.attack_speed + aspd)/100))
    /// if self.targets > 1 and self.module == 2:
    /// dps += extradmg/(self.atk_interval/((self.attack_speed + aspd)/100)) * (self.targets -1)
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

        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut fallouttime: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut eledmg: f64 = 0.0;
        let mut ele_gauge: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut eledps: f64 = 0.0;
        let mut bonusdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut extradmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        aspd = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 2.0 {
            30.0
        } else {
            0.0
        };
        atk_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        eledmg = 0.0;
        let mut bonus_scale =
            if (self.unit.targets as f64) == 1.0 && ((self.unit.elite as f64) as f64) == 2.0 {
                self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
            } else {
                0.0
            };
        eledmg = if ((self.unit.module_index as f64) as f64) == 3.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && self.unit.talent2_damage
        {
            ((self.unit.module_level as f64) as f64) * 0.1 / (1.0 + self.unit.buff_fragile)
        } else {
            0.0
        };
        let mut extra_scale = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            self.unit.talent2_parameters.get(3).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            atk_interval = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                (self.unit.attack_interval as f64)
                    * self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                (self.unit.attack_interval as f64)
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus_scale * 0.05) as f64);
            extradmg = ((final_atk * extra_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * extra_scale * 0.05) as f64);
            dps = hitdmg / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0));
            if (self.unit.module_index as f64) == 3.0 {
                ele_gauge = if self.unit.module_damage {
                    1000.0
                } else {
                    2000.0
                };
                eledps = dps * 0.08;
                fallouttime = ele_gauge / eledps;
                dps += 12000.0 / (fallouttime + 15.0) / (1.0 + self.unit.buff_fragile);
                dps += eledmg * final_atk
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * 15.0
                    / (fallouttime + 15.0);
            }
            if (self.unit.targets as f64) == 1.0 {
                dps += bonusdmg / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0));
            }
            if (self.unit.targets as f64) > 1.0 && (self.unit.module_index as f64) == 2.0 {
                dps += extradmg / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0))
                    * ((self.unit.targets as f64) - 1.0);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            if self.unit.talent_damage {
                atk_scale *= self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            }
            aspd += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus_scale * 0.05) as f64);
            extradmg = ((final_atk * extra_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * extra_scale * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
            if (self.unit.module_index as f64) == 3.0 {
                ele_gauge = if self.unit.module_damage {
                    1000.0
                } else {
                    2000.0
                };
                eledps = dps * 0.08;
                fallouttime = ele_gauge / eledps;
                dps += 12000.0 / (fallouttime + 15.0) / (1.0 + self.unit.buff_fragile);
                dps += eledmg * final_atk
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * 15.0
                    / (fallouttime + 15.0);
            }
            if (self.unit.targets as f64) == 1.0 {
                dps += bonusdmg
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0));
            }
            if (self.unit.targets as f64) > 1.0 && (self.unit.module_index as f64) == 2.0 {
                dps += extradmg
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * ((self.unit.targets as f64) - 1.0);
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Ebenholz {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ebenholz {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ebenholz {
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
