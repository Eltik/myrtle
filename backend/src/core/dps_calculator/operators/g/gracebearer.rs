//! DPS calculations for Gracebearer
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Gracebearer operator implementation
pub struct Gracebearer {
    pub unit: OperatorUnit,
}

impl Gracebearer {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Gracebearer operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            1, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atkbuff = 0
    /// if self.elite > 0:
    /// atkbuff = self.talent1_params[0] + self.talent1_params[1] if self.talent_dmg else self.talent1_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// atk_scale = self.skill_params[0] if self.skill == 1 else 1
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100
    /// if self.skill == 1:
    /// dps *= 2
    /// if self.skill_dmg:
    /// ele_gauge = 1000 if self.trait_dmg else 2000
    /// time_to_fallout = ele_gauge/(0.1 * dps)
    /// dps += 6000/(10 + time_to_fallout)
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) if not self.skill_dmg else final_atk * skill_scale
    /// dps += 3 * skilldmg/((self.skill_cost + 1.2)/(1+self.sp_boost)) * min(self.targets, self.skill_params[1])
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

        let mut time_to_fallout: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut ele_gauge: f64 = 0.0;

        atkbuff = 0.0;
        if (self.unit.elite as f64) > 0.0 {
            atkbuff = if self.unit.talent_damage {
                self.unit.talent1_parameters[0] + self.unit.talent1_parameters[1]
            } else {
                self.unit.talent1_parameters[0]
            };
        }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        atk_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters[0]
        } else {
            1.0
        };
        hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        if (self.unit.skill_index as f64) == 1.0 {
            dps *= 2.0;
            if self.unit.skill_damage {
                ele_gauge = if self.unit.trait_damage {
                    1000.0
                } else {
                    2000.0
                };
                time_to_fallout = ele_gauge / (0.1 * dps);
                dps += 6000.0 / (10.0 + time_to_fallout);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[0];
            skilldmg = if !self.unit.skill_damage {
                ((final_atk * skill_scale - defense) as f64)
                    .max((final_atk * skill_scale * 0.05) as f64)
            } else {
                final_atk * skill_scale
            };
            dps += 3.0 * skilldmg
                / (((self.unit.skill_cost as f64) + 1.2) / (1.0 + (self.unit.sp_boost as f64)))
                * ((self.unit.targets as f64) as f64).min((self.unit.skill_parameters[1]) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Gracebearer {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Gracebearer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
