//! DPS calculations for Leto
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Leto operator implementation
pub struct Leto {
    pub unit: OperatorUnit,
}

impl Leto {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Leto operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// atk_scale = 0.8 if self.skill < 2 and not self.trait_dmg else 1
    /// aspd = 12 if self.module == 2 and (self.targets > 1 or self.module_dmg) else 0
    /// aspd += self.talent1_params[0]
    /// final_atk = self.atk * (1 + self.skill_params[0] * min(self.skill,1) + self.buff_atk) + self.buff_atk_flat
    /// if self.skill == 1: aspd += self.skill_params[1]
    /// hitdmg = np.fmax(final_atk *atk_scale - defense, final_atk* atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 2 and self.targets > 1: dps *= 2
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

        let mut atk_scale =
            if ((self.unit.skill_index as f64) as f64) < 2.0 && !self.unit.trait_damage {
                0.8
            } else {
                1.0
            };
        let mut aspd = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.targets as f64) > 1.0 || self.unit.module_damage)
        {
            12.0
        } else {
            0.0
        };
        aspd += self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        let mut final_atk = self.unit.atk
            * (1.0
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                    * ((self.unit.skill_index as f64) as f64).min((1) as f64)
                + self.unit.buff_atk)
            + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 1.0 {
            aspd += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
        }
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut dps =
            hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        if (self.unit.skill_index as f64) == 2.0 && (self.unit.targets as f64) > 1.0 {
            dps *= 2.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Leto {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Leto {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
