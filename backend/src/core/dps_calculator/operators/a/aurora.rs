//! DPS calculations for Aurora
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Aurora operator implementation
pub struct Aurora {
    pub unit: OperatorUnit,
}

impl Aurora {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("skill", "1/3vsFreeze", false, &[2], &[], 0, 0)];

    /// Creates a new Aurora operator
    #[allow(unused_parens)]
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
    /// atk_interval = 1.85 if self.skill == 2 else self.atk_interval
    /// atkbuff = self.skill_params[0] if self.skill == 2 else 0
    /// skill_scale = self.skill_params[3]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg =  np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// avgdmg = hitdmg
    /// if self.skill_dmg and self.skill == 2: avgdmg = 2/3 * hitdmg + 1/3 * skilldmg
    /// dps = avgdmg/atk_interval * self.attack_speed/100
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atk_interval = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            1.85
        } else {
            (self.unit.attack_interval as f64)
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut skill_scale = self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut skilldmg = ((final_atk * skill_scale - defense) as f64)
            .max((final_atk * skill_scale * 0.05) as f64);
        let mut avgdmg = hitdmg;
        if self.unit.skill_damage && (self.unit.skill_index as f64) == 2.0 {
            avgdmg = 2.0 / 3.0 * hitdmg + 1.0 / 3.0 * skilldmg;
        }
        let mut dps = avgdmg / atk_interval * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Aurora {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Aurora {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Aurora {
    fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        Self::skill_dps(self, enemy)
    }

    fn unit(&self) -> &OperatorUnit {
        &self.unit
    }

    fn unit_mut(&mut self) -> &mut OperatorUnit {
        &mut self.unit
    }
}
