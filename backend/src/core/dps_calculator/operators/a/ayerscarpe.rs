//! DPS calculations for Ayerscarpe
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Ayerscarpe operator implementation
pub struct Ayerscarpe {
    pub unit: OperatorUnit,
}

impl Ayerscarpe {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Ayerscarpe operator
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
    /// atk_scale = 0.8 if not self.trait_dmg and self.skill == 1 else 1
    /// bonus = 0.1 if self.module == 1 else 0
    /// aspd = self.talent1_params[0] if self.elite > 0 else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// targets = self.skill_params[2] if self.skill == 1 else 1
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale *(1-res/100), final_atk * skill_scale * 0.05) * min(self.targets,targets)
    /// avgdmg = (self.skill_cost * hitdmg + skilldmg)/(self.skill_cost + 1)
    /// if self.skill == 0: avgdmg = hitdmg
    /// dps = (avgdmg + bonusdmg) / self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// skilldmg *= self.targets if self.trait_dmg else self.targets -1
    /// dps = (hitdmg + bonusdmg + skilldmg) / self.atk_interval * (self.attack_speed+aspd)/100
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

        let mut hitdmg: f64 = 0.0;
        let mut bonusdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        atk_scale = if !self.unit.trait_damage && ((self.unit.skill_index as f64) as f64) == 1.0 {
            0.8
        } else {
            1.0
        };
        let mut bonus = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        aspd = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut targets = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((targets) as f64);
            avgdmg = ((self.unit.skill_cost as f64) * hitdmg + skilldmg)
                / ((self.unit.skill_cost as f64) + 1.0);
            if (self.unit.skill_index as f64) == 0.0 {
                avgdmg = hitdmg;
            }
            dps = (avgdmg + bonusdmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            skilldmg *= if self.unit.trait_damage {
                (self.unit.targets as f64)
            } else {
                (self.unit.targets as f64) - 1.0
            };
            dps = (hitdmg + bonusdmg + skilldmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Ayerscarpe {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ayerscarpe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ayerscarpe {
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
