//! DPS calculations for Aak
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Aak operator implementation
pub struct Aak {
    pub unit: OperatorUnit,
}

impl Aak {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[];

    /// Creates a new Aak operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// cdmg = max(self.talent1_params)
    /// crate = 0.25
    /// if self.module == 1 and self.module_lvl > 1: crate = 0.25 + 0.75 * 0.2 if self.module_lvl == 2 else 0.25 + 0.75 * 0.3
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat if self.skill == 3 else self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// aspd = self.skill_params[0] if self.skill == 1 else self.skill_params[1] * self.skill / 3
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// avghit = (1-crate) * hitdmg + crate * critdmg
    /// dps = avghit/self.atk_interval * (self.attack_speed + aspd)/100
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

        let mut cdmg = self
            .unit
            .talent1_parameters
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let mut crit_rate = 0.25;
        if (self.unit.module_index as f64) == 1.0 && (self.unit.module_level as f64) > 1.0 {
            crit_rate = if ((self.unit.module_level as f64) as f64) == 2.0 {
                0.25 + 0.75 * 0.2
            } else {
                0.25 + 0.75 * 0.3
            };
        }
        let mut final_atk = if ((self.unit.skill_index as f64) as f64) == 3.0 {
            self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat
        } else {
            self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat
        };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                * ((self.unit.skill_index as f64) as f64)
                / 3.0
        };
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut critdmg =
            ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
        let mut avghit = (1.0 - crit_rate) * hitdmg + crit_rate * critdmg;
        let mut dps =
            avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Aak {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Aak {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Aak {
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
