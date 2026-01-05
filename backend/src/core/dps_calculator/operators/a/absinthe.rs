//! DPS calculations for Absinthe
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Absinthe operator implementation
pub struct Absinthe {
    pub unit: OperatorUnit,
}

impl Absinthe {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Absinthe operator
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
    /// dmg_scale = self.talent1_params[1] if self.talent_dmg and self.elite > 0 else 1
    /// newres = np.fmax(0,res-10) if self.module == 1 else res
    /// final_atk = self.atk * (1 + self.skill_params[0] + self.buff_atk) + self.buff_atk_flat if self.skill == 1 else self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// atk_scale = 4 * self.skill_params[1] if self.skill == 2 else 1
    /// hitdmgarts = np.fmax(final_atk * atk_scale *(1-newres/100), final_atk * atk_scale * 0.05) * dmg_scale
    /// dps = hitdmgarts/self.atk_interval * self.attack_speed/100
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

        let mut dmg_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut newres = if ((self.unit.module_index as f64) as f64) == 1.0 {
            ((0) as f64).max((res - 10.0) as f64)
        } else {
            res
        };
        let mut final_atk = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.atk
                * (1.0
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                    + self.unit.buff_atk)
                + self.unit.buff_atk_flat
        } else {
            self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat
        };
        let mut atk_scale = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            4.0 * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut hitdmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
            .max((final_atk * atk_scale * 0.05) as f64)
            * dmg_scale;
        let mut dps =
            hitdmgarts / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Absinthe {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Absinthe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
