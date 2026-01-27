//! DPS calculations for Andreana
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Andreana operator implementation
pub struct Andreana {
    pub unit: OperatorUnit,
}

impl Andreana {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("module", "atMaxRange", false, &[], &[1], 0, 0)];

    /// Creates a new Andreana operator
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
    /// atk_scale = 1.15 if self.module == 1 and self.module_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * min(self.skill,1)) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk *atk_scale - defense, final_atk* atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + self.talent1_params[0])/100
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

        let mut atk_scale =
            if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
                1.15
            } else {
                1.0
            };
        let mut final_atk = self.unit.atk
            * (1.0
                + self.unit.buff_atk
                + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                    * ((self.unit.skill_index as f64) as f64).min((1) as f64))
            + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64)
            * (self.unit.attack_speed
                + self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0))
            / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Andreana {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Andreana {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Andreana {
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
