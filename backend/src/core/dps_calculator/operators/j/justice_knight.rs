//! DPS calculations for JusticeKnight
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// JusticeKnight operator implementation
pub struct JusticeKnight {
    pub unit: OperatorUnit,
}

impl JusticeKnight {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new JusticeKnight operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            0, // default_skill_index
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
    /// fragile = self.talent1_params[1] - 1
    /// if not self.talent2_dmg: fragile = 0
    /// fragile = max(fragile, self.buff_fragile)
    /// final_atk = self.atk * (1  + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * (1 + fragile)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 /(1+self.buff_fragile)
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

        let mut fragile = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0) - 1.0;
        if !self.unit.talent2_damage {
            fragile = 0.0;
        }
        fragile = ((fragile) as f64).max((self.unit.buff_fragile) as f64);
        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * (1.0 + fragile);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed
            / 100.0
            / (1.0 + self.unit.buff_fragile);
        return dps;
    }
}

impl std::ops::Deref for JusticeKnight {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for JusticeKnight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
