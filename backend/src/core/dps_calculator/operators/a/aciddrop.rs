//! DPS calculations for Aciddrop
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Aciddrop operator implementation
pub struct Aciddrop {
    pub unit: OperatorUnit,
}

impl Aciddrop {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Aciddrop operator
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
    /// if self.elite == 0: mindmg = 0.05
    /// elif self.talent_dmg: mindmg = self.talent1_params[1]
    /// else: mindmg = self.talent1_params[0]
    /// aspd = self.skill_params[0] if self.skill == 1 else 0
    /// atkbuff = self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * mindmg)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * max(1, self.skill)
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

        let mut mindmg: f64 = 0.0;
        if (self.unit.elite as f64) == 0.0 {
            mindmg = 0.05;
        } else if self.unit.talent_damage {
            mindmg = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        } else {
            mindmg = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        }
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * mindmg) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0
            * ((1) as f64).max((self.unit.skill_index as f64) as f64);
        return dps;
    }
}

impl std::ops::Deref for Aciddrop {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Aciddrop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
