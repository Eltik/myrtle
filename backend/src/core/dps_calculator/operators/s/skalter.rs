//! DPS calculations for Skalter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Skalter operator implementation
pub struct Skalter {
    pub unit: OperatorUnit,
}

impl Skalter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("module", "noModBonus", true, &[], &[1], 0, 0),
        ("talent", "+Seaborn", false, &[], &[], 0, 0),
        ("talent2", "AllyInRange(add+9%forAH)", false, &[], &[], 0, 0),
    ];

    /// Creates a new Skalter operator
    #[allow(unused_parens)]
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
    /// if self.skill != 3: return res * 0
    /// atkbuff = 0.08 if self.module == 1 and self.module_dmg else 0
    /// if self.talent2_dmg: atkbuff += self.talent2_params[0]
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// dps = final_atk * skill_scale * np.fmax(1,-defense) * self.targets
    /// if self.talent_dmg: dps *= 2
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

        if (self.unit.skill_index as f64) != 3.0 {
            return res * 0.0;
        }
        let mut atkbuff =
            if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
                0.08
            } else {
                0.0
            };
        if self.unit.talent2_damage {
            atkbuff += self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0);
        }
        let mut skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut dps = final_atk
            * skill_scale
            * ((1) as f64).max((-defense) as f64)
            * (self.unit.targets as f64);
        if self.unit.talent_damage {
            dps *= 2.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Skalter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Skalter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Skalter {
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
