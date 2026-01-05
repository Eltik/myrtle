//! DPS calculations for Sharp
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Sharp operator implementation
pub struct Sharp {
    pub unit: OperatorUnit,
}

impl Sharp {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Sharp operator
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
    /// final_atk = self.atk * (1 + self.buff_atk + self.talent1_params[0]) + self.buff_atk_flat
    /// skill_scale = self.skill_params[1] if self.skill == 1 else 1
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// dps =  hitdmg / self.atk_interval * (self.attack_speed) / 100
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

        let mut final_atk = self.unit.atk
            * (1.0
                + self.unit.buff_atk
                + self.unit.talent1_parameters.first().copied().unwrap_or(0.0))
            + self.unit.buff_atk_flat;
        let mut skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut hitdmg = ((final_atk * skill_scale - defense) as f64)
            .max((final_atk * skill_scale * 0.05) as f64);
        let mut dps =
            hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Sharp {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Sharp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
