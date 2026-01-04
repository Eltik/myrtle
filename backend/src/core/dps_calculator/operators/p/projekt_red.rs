//! DPS calculations for ProjektRed
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// ProjektRed operator implementation
pub struct ProjektRed {
    pub unit: OperatorUnit,
}

impl ProjektRed {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new ProjektRed operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atkbuff = 0.1 if self.module_dmg and self.module == 2 else 0
    /// mindmg = 0.05 if self.elite == 0 else self.talent1_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + self.skill_params[0]*self.skill) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * mindmg)
    /// dps = hitdmg / self.atk_interval * self.attack_speed/100
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

        let mut atkbuff =
            if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 2.0 {
                0.1
            } else {
                0.0
            };
        let mut mindmg = if ((self.unit.elite as f64) as f64) == 0.0 {
            0.05
        } else {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        };
        let mut final_atk = self.unit.atk
            * (1.0
                + atkbuff
                + self.unit.buff_atk
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                    * (self.unit.skill_index as f64))
            + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * mindmg) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for ProjektRed {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ProjektRed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
