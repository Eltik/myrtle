//! DPS calculations for Pith
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Pith operator implementation
pub struct Pith {
    pub unit: OperatorUnit,
}

impl Pith {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[];

    /// Creates a new Pith operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// newres = np.fmax(0, res - self.talent1_params[0])
    /// atkbuff = self.skill_params[0] * self.skill
    /// aspd = self.skill_params[1] * self.skill
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-newres/100), final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100
    /// return dps*self.targets
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut newres = ((0) as f64).max((res - self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)) as f64);
        let mut atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) * (self.unit.skill_index as f64);
        let mut aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0) * (self.unit.skill_index as f64);
        let mut final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk * (1.0 -newres/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        return dps*(self.unit.targets as f64);
    }
}

impl std::ops::Deref for Pith {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Pith {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Pith {
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
