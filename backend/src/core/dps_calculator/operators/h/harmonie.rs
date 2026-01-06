//! DPS calculations for Harmonie
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Harmonie operator implementation
pub struct Harmonie {
    pub unit: OperatorUnit,
}

impl Harmonie {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[("talent", "vsBlocked", false, &[2], &[], 1, 0)];

    /// Creates a new Harmonie operator
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
    /// atk_scale = self.talent1_params[0] if self.elite > 0 and self.talent_dmg or self.skill == 2 else 1
    /// if self.skill < 2:
    /// atk_interval = self.atk_interval/5 if self.skill == 1 else self.atk_interval
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale * atk_scale * (1-res/100), final_atk * skill_scale * atk_scale * 0.05)
    /// dps = hitdmg / atk_interval * self.attack_speed / 100
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval * self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk  * atk_scale * 0.05)
    /// extra_dps = np.fmax(self.skill_params[0] * (1-res/100), self.skill_params[0] * 0.05) * self.targets
    /// dps = hitdmg / atk_interval * self.attack_speed / 100 + extra_dps
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
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        atk_scale = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage
            || ((self.unit.skill_index as f64) as f64) == 2.0
        {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atk_interval = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                (self.unit.attack_interval as f64) / 5.0
            } else {
                (self.unit.attack_interval as f64)
            };
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            dps = hitdmg / atk_interval * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = (self.unit.attack_interval as f64)
                * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut extra_dps = ((self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * (1.0 - res / 100.0)) as f64)
                .max((self.unit.skill_parameters.first().copied().unwrap_or(0.0) * 0.05) as f64)
                * (self.unit.targets as f64);
            dps = hitdmg / atk_interval * self.unit.attack_speed / 100.0 + extra_dps;
        }
        return dps;
    }
}

impl std::ops::Deref for Harmonie {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Harmonie {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Harmonie {
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
