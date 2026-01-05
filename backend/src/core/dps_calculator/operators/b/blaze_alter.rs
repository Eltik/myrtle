//! DPS calculations for BlazeAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// BlazeAlter operator implementation
pub struct BlazeAlter {
    pub unit: OperatorUnit,
}

impl BlazeAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new BlazeAlter operator
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
    /// burst_scale = 1.1 if self.module == 1 and self.skill_dmg else 1
    /// falloutdmg = 7000
    /// atkbuff = self.skill_params[0] if self.skill == 2 else 0
    /// module_atk = 0.05 * (self.module_lvl - 1) if self.module == 1 and self.module_lvl > 1 and ((self.trait_dmg and self.skill != 3) or (self.skill_dmg and self.skill == 3)) else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// if self.elite > 0: falloutdmg += final_atk * self.talent1_params[0]
    ///
    /// if self.skill == 0:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + module_atk) + self.buff_atk_flat
    /// newres = np.fmax(0,res-20)
    /// elegauge = 1000 if self.skill_dmg else 2000
    /// hitdmg1 = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk * (1-newres/100), final_atk * 0.05) #number 2 is against enemies under burn fallout
    /// skilldmg1 = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// skilldmg2 = np.fmax(final_atk * skill_scale * (1-newres/100), final_atk * skill_scale * 0.05)
    /// dpsNorm = hitdmg1/self.atk_interval * (self.attack_speed)/100 + skilldmg1 * self.targets
    /// dpsFallout = hitdmg2/self.atk_interval * (self.attack_speed)/100 + skilldmg2 * self.targets
    /// timeToFallout = elegauge/(skilldmg1 * self.skill_params[1])
    /// dps = (dpsNorm * timeToFallout + dpsFallout * burst_scale * 10 + falloutdmg)/(timeToFallout + 10)
    /// if not self.trait_dmg: dps = dpsNorm
    ///
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// skill_scale = self.skill_params[2]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff + module_atk) + self.buff_atk_flat
    /// newres = np.fmax(0,res-20)
    /// elegauge = 1000 if self.skill_dmg else 2000
    /// hitdmg1 = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * min(self.targets,3)
    /// hitdmg2 = np.fmax(final_atk * (1-newres/100), final_atk * 0.05) * min(self.targets,3) #number 2 is against enemies under burn fallout
    /// skilldmg1 = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// skilldmg2 = np.fmax(final_atk * skill_scale * (1-newres/100), final_atk * skill_scale * 0.05)
    /// dpsNorm = hitdmg1/2.5 * (self.attack_speed)/100 + skilldmg1 * self.targets
    /// dpsFallout = hitdmg2/2.5 * (self.attack_speed)/100 + skilldmg2 * self.targets
    /// timeToFallout = elegauge/(skilldmg1 * self.skill_params[1])
    /// dps = (dpsNorm * timeToFallout + dpsFallout * burst_scale * 10 + falloutdmg)/(timeToFallout + 10)
    /// if not self.trait_dmg: dps = dpsNorm
    ///
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + module_atk) + self.buff_atk_flat
    /// newres = np.fmax(0,res-20) if self.skill_dmg else res
    /// ele_scale = self.skill_params[3] if self.skill_dmg else 0
    /// hitdmg = np.fmax(final_atk * (1-newres/100), final_atk * 0.05) + final_atk * ele_scale
    /// dps = hitdmg * burst_scale / 0.3 * self.attack_speed/ 100 * self.targets
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

        let mut newres: f64 = 0.0;
        let mut hitdmg1: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut skill_scale: f64 = 0.0;
        let mut dps_norm: f64 = 0.0;
        let mut hitdmg2: f64 = 0.0;
        let mut time_to_fallout: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut ele_scale: f64 = 0.0;
        let mut skilldmg1: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut elegauge: f64 = 0.0;
        let mut dps_fallout: f64 = 0.0;
        let mut skilldmg2: f64 = 0.0;
        let mut burst_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        burst_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.skill_damage {
            1.1
        } else {
            1.0
        };
        let mut falloutdmg = 7000.0;
        atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut module_atk = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && ((self.unit.trait_damage && ((self.unit.skill_index as f64) as f64) != 3.0)
                || (self.unit.skill_damage && ((self.unit.skill_index as f64) as f64) == 3.0))
        {
            0.05 * (((self.unit.module_level as f64) as f64) - 1.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        if (self.unit.elite as f64) > 0.0 {
            falloutdmg += final_atk * self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        }
        if (self.unit.skill_index as f64) == 0.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + module_atk) + self.unit.buff_atk_flat;
            newres = ((0) as f64).max((res - 20.0) as f64);
            elegauge = if self.unit.skill_damage {
                1000.0
            } else {
                2000.0
            };
            hitdmg1 = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            hitdmg2 = ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            skilldmg1 = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            skilldmg2 = ((final_atk * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps_norm = hitdmg1 / (self.unit.attack_interval as f64) * (self.unit.attack_speed)
                / 100.0
                + skilldmg1 * (self.unit.targets as f64);
            dps_fallout = hitdmg2 / (self.unit.attack_interval as f64) * (self.unit.attack_speed)
                / 100.0
                + skilldmg2 * (self.unit.targets as f64);
            time_to_fallout =
                elegauge / (skilldmg1 * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0));
            dps = (dps_norm * time_to_fallout + dps_fallout * burst_scale * 10.0 + falloutdmg)
                / (time_to_fallout + 10.0);
            if !self.unit.trait_damage {
                dps = dps_norm;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff + module_atk)
                + self.unit.buff_atk_flat;
            newres = ((0) as f64).max((res - 20.0) as f64);
            elegauge = if self.unit.skill_damage {
                1000.0
            } else {
                2000.0
            };
            hitdmg1 = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((3) as f64);
            hitdmg2 = ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((3) as f64);
            skilldmg1 = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            skilldmg2 = ((final_atk * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps_norm = hitdmg1 / 2.5 * (self.unit.attack_speed) / 100.0
                + skilldmg1 * (self.unit.targets as f64);
            dps_fallout = hitdmg2 / 2.5 * (self.unit.attack_speed) / 100.0
                + skilldmg2 * (self.unit.targets as f64);
            time_to_fallout =
                elegauge / (skilldmg1 * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0));
            dps = (dps_norm * time_to_fallout + dps_fallout * burst_scale * 10.0 + falloutdmg)
                / (time_to_fallout + 10.0);
            if !self.unit.trait_damage {
                dps = dps_norm;
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk + module_atk)
                + self.unit.buff_atk_flat;
            newres = if self.unit.skill_damage {
                ((0) as f64).max((res - 20.0) as f64)
            } else {
                res
            };
            ele_scale = if self.unit.skill_damage {
                self.unit.skill_parameters.get(3).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            hitdmg = ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64)
                + final_atk * ele_scale;
            dps = hitdmg * burst_scale / 0.3 * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// if self.skill == 3:
    /// return(self.skill_dps(defense,res) * self.skill_params[2] * (0.3/(self.attack_speed/100)))
    /// else:
    /// return(super().total_dmg(defense,res))
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
            // UNTRANSLATED: return(self.skill_dps(defense,res) * self.skill_params[2] * (0.3/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(super().total_dmg(defense,res)) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for BlazeAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for BlazeAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for BlazeAlter {
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
