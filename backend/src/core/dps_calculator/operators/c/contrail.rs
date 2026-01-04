//! DPS calculations for Contrail
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Contrail operator implementation
pub struct Contrail {
    pub unit: OperatorUnit,
}

impl Contrail {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Contrail operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// targets = 3 if self.skill == 2 else 1
    /// atk_scale = self.talent1_params[0] if self.elite > 0 else 1
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100 * min(self.targets, targets)
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

        let mut targets = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            3.0
        } else {
            1.0
        };
        let mut atk_scale = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
            * ((self.unit.targets as f64) as f64).min((targets) as f64);
        return dps;
    }
}

impl std::ops::Deref for Contrail {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Contrail {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
