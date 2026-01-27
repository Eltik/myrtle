//! DPS calculations for Underflow
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Underflow operator implementation
pub struct Underflow {
    pub unit: OperatorUnit,
}

impl Underflow {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("talent", "vsSeaborn", false, &[], &[], 1, 0)];

    /// Creates a new Underflow operator
    #[allow(unused_parens)]
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
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// aspd = self.skill_params[3] if self.skill == 2 else 0
    /// targets = self.skill_params[4] if self.skill == 2 else 1
    /// arts_dmg = self.talent1_params[2] if self.elite > 0 else 0
    /// if self.talent_dmg: arts_dmg *= 2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * min(self.targets,targets)
    /// dps += np.fmax(arts_dmg * (1 - res/100), arts_dmg * 0.05) * min(self.targets,targets)
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

        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(3).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut targets = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(4).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut arts_dmg = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if self.unit.talent_damage {
            arts_dmg *= 2.0;
        }
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0
            * ((self.unit.targets as f64) as f64).min((targets) as f64);
        dps += ((arts_dmg * (1.0 - res / 100.0)) as f64).max((arts_dmg * 0.05) as f64)
            * ((self.unit.targets as f64) as f64).min((targets) as f64);
        return dps;
    }
}

impl std::ops::Deref for Underflow {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Underflow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Underflow {
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
