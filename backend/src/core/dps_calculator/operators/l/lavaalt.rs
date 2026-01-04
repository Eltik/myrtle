//! DPS calculations for Lavaalt
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Lavaalt operator implementation
pub struct Lavaalt {
    pub unit: OperatorUnit,
}

impl Lavaalt {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Lavaalt operator
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
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmgarts/self.atk_interval * self.attack_speed/100 * self.targets
    /// if self.skill_dmg and self.targets > 1 and self.skill == 1:
    /// dps *= 2
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmgarts * self.targets
    /// if self.skill_dmg:
    /// dps *= 2
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

        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut dps: f64 = 0.0;

        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64))
                + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmgarts / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
            if self.unit.skill_damage
                && (self.unit.targets as f64) > 1.0
                && (self.unit.skill_index as f64) == 1.0
            {
                dps *= 2.0;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmgarts * (self.unit.targets as f64);
            if self.unit.skill_damage {
                dps *= 2.0;
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Lavaalt {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Lavaalt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
