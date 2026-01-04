//! DPS calculations for Ashlock
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Ashlock operator implementation
pub struct Ashlock {
    pub unit: OperatorUnit,
}

impl Ashlock {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Ashlock operator
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
    /// atkbuff = self.talent1_params[1] if self.talent_dmg else self.talent1_params[0]
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)
    /// atk_interval = self.atk_interval if self.skill != 2 else self.atk_interval * (1 + self.skill_params[1])
    /// dps = hitdmg / atk_interval * self.attack_speed/100 * self.targets
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

        let mut atkbuff = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        };
        let mut atk_scale =
            if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
                1.1
            } else {
                1.0
            };
        atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk + atkbuff)
            + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut atk_interval = if ((self.unit.skill_index as f64) as f64) != 2.0 {
            (self.unit.attack_interval as f64)
        } else {
            (self.unit.attack_interval as f64)
                * (1.0 + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
        };
        let mut dps =
            hitdmg / atk_interval * self.unit.attack_speed / 100.0 * (self.unit.targets as f64);
        return dps;
    }
}

impl std::ops::Deref for Ashlock {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ashlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
