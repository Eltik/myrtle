//! DPS calculations for Indra
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Indra operator implementation
pub struct Indra {
    pub unit: OperatorUnit,
}

impl Indra {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Indra operator
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
    /// aspd = 10 if self.module_dmg and self.module == 1 else 0
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// newdef = defense * (1 - self.skill_params[1])
    /// if self.skill == 1: final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// skilldmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// dps = 0.2*(4*hitdmg + skilldmg)/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
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

        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        aspd = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 1.0 {
            10.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut newdef = defense * (1.0 - self.unit.skill_parameters[1]);
            if (self.unit.skill_index as f64) == 1.0 {
                final_atk = self.unit.atk
                    * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[0])
                    + self.unit.buff_atk_flat;
            }
            skilldmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
            dps = 0.2 * (4.0 * hitdmg + skilldmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[0])
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Indra {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Indra {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
