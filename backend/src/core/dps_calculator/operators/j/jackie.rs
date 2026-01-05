//! DPS calculations for Jackie
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Jackie operator implementation
pub struct Jackie {
    pub unit: OperatorUnit,
}

impl Jackie {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Jackie operator
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
    /// aspd = self.talent1_params[1] if self.talent_dmg else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * self.skill_params[0] - defense, final_atk * self.skill_params[0] * 0.05)
    /// avgdmg = (hitdmg * self.skill_cost + skilldmg) / (self.skill_cost+1) if self.skill == 1 else hitdmg
    /// dps = avgdmg/self.atk_interval*(self.attack_speed+aspd)/100
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

        let mut aspd = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut skilldmg = ((final_atk * self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            - defense) as f64)
            .max(
                (final_atk * self.unit.skill_parameters.first().copied().unwrap_or(0.0) * 0.05)
                    as f64,
            );
        let mut avgdmg = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            (hitdmg * (self.unit.skill_cost as f64) + skilldmg)
                / ((self.unit.skill_cost as f64) + 1.0)
        } else {
            hitdmg
        };
        let mut dps =
            avgdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Jackie {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Jackie {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
