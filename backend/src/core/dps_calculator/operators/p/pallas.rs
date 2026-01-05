//! DPS calculations for Pallas
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Pallas operator implementation
pub struct Pallas {
    pub unit: OperatorUnit,
}

impl Pallas {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Pallas operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// atk_scale = 1
    /// atkbuff = min(self.talent1_params) if self.talent_dmg and self.elite > 0 else 0
    /// if self.trait_dmg:
    /// atk_scale = 1.3 if self.module == 1 else 1.2
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk* atk_scale * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + (1 + self.skill) * skillhitdmg) / (sp_cost + 1)
    /// dps = avgphys/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 3:
    /// if self.skill_dmg:
    /// atkbuff = max(atkbuff, self.skill_params[2]) #vigor doesnt stack
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100     * min(self.targets, 3)
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

        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;

        atk_scale = 1.0;
        atkbuff = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit
                .talent1_parameters
                .iter()
                .cloned()
                .fold(f64::INFINITY, f64::min)
        } else {
            0.0
        };
        if self.unit.trait_damage {
            atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
                1.3
            } else {
                1.2
            };
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            avgphys = (sp_cost * hitdmg + (1.0 + (self.unit.skill_index as f64)) * skillhitdmg)
                / (sp_cost + 1.0);
            dps = avgphys / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            if self.unit.skill_damage {
                atkbuff = ((atkbuff) as f64)
                    .max((self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)) as f64);
            }
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Pallas {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Pallas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
