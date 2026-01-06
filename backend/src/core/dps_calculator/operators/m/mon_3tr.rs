//! DPS calculations for Mon3tr
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Mon3tr operator implementation
pub struct Mon3tr {
    pub unit: OperatorUnit,
}

impl Mon3tr {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Mon3tr operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// aspd = self.talent2_params[1] if self.elite > 1 else 0
    /// if self.skill < 3: return res * 0
    /// if self.skill == 3:
    /// atk_interval = self.atk_interval + self.skill_params[4]
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] ) + self.buff_atk_flat
    /// dps = final_atk/(atk_interval / (self.attack_speed+aspd)*100) * np.fmax(-defense, 1) * min(self.targets, 3)
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

        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;

        aspd = if ((self.unit.elite as f64) as f64) > 1.0 {
            self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 3.0 {
            return res * 0.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64)
                + self.unit.skill_parameters.get(4).copied().unwrap_or(0.0);
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            dps = final_atk / (atk_interval / (self.unit.attack_speed + aspd) * 100.0)
                * ((-defense) as f64).max((1) as f64)
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Mon3tr {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Mon3tr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Mon3tr {
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
