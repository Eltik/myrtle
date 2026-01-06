//! DPS calculations for Angelina
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Angelina operator implementation
pub struct Angelina {
    pub unit: OperatorUnit,
}

impl Angelina {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Angelina operator
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
    /// aspd = self.talent1_params[0]
    /// if self.module == 1:
    /// if self.module_lvl == 2: aspd += 3
    /// if self.module_lvl == 3: aspd += 5
    ///
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/ 100
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval * 0.15
    /// skill_scale = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// dps = hitdmg / atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 3:
    /// targets = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/ 100 * min(self.targets, targets)
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
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        aspd = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        if (self.unit.module_index as f64) == 1.0 {
            if (self.unit.module_level as f64) == 2.0 {
                aspd += 3.0;
            }
            if (self.unit.module_level as f64) == 3.0 {
                aspd += 5.0;
            }
        }
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = (self.unit.attack_interval as f64) * 0.15;
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps = hitdmg / atk_interval * (self.unit.attack_speed + aspd) / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut targets = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((targets) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Angelina {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Angelina {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Angelina {
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
