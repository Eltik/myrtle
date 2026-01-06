//! DPS calculations for MissChristine
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// MissChristine operator implementation
pub struct MissChristine {
    pub unit: OperatorUnit,
}

impl MissChristine {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new MissChristine operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// atk_scale = 1.1 if self.module == 1 and self.trait_dmg else 1
    /// if self.module == 1 and self.module_lvl > 1 and self.module_dmg:
    /// atk_scale *= 0.9 + 0.1 * self.module_lvl
    /// if self.skill == 0:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100
    /// if self.skill == 1:
    /// final_atk = self.atk * (1 + self.skill_params[0] + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100
    /// if self.skill_dmg:
    /// ele_gauge = 1000 if self.trait_dmg else 2000
    /// time_to_fallout = ele_gauge/(0.1 * dps)
    /// dps += 6000/(10 + time_to_fallout)
    /// dps *= min(2,self.targets)
    ///
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[5]
    /// ele_scale = self.skill_params[2] if self.skill_dmg and self.trait_dmg else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale * atk_scale * (1-res/100), final_atk * skill_scale * atk_scale * 0.05)
    /// eledmg = final_atk * ele_scale * atk_scale
    /// dps = (hitdmg + eledmg) * self.targets
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

        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut ele_scale: f64 = 0.0;
        let mut time_to_fallout: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut ele_gauge: f64 = 0.0;
        let mut eledmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.trait_damage {
            1.1
        } else {
            1.0
        };
        if (self.unit.module_index as f64) == 1.0
            && (self.unit.module_level as f64) > 1.0
            && self.unit.module_damage
        {
            atk_scale *= 0.9 + 0.1 * (self.unit.module_level as f64);
        }
        if (self.unit.skill_index as f64) == 0.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                    + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if self.unit.skill_damage {
                ele_gauge = if self.unit.trait_damage {
                    1000.0
                } else {
                    2000.0
                };
                time_to_fallout = ele_gauge / (0.1 * dps);
                dps += 6000.0 / (10.0 + time_to_fallout);
            }
            dps *= ((2) as f64).min((self.unit.targets as f64) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(5).copied().unwrap_or(0.0);
            ele_scale = if self.unit.skill_damage && self.unit.trait_damage {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            eledmg = final_atk * ele_scale * atk_scale;
            dps = (hitdmg + eledmg) * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for MissChristine {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for MissChristine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for MissChristine {
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
