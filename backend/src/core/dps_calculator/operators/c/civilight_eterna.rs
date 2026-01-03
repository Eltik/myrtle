//! DPS calculations for CivilightEterna
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// CivilightEterna operator implementation
pub struct CivilightEterna {
    pub unit: OperatorUnit,
}

impl CivilightEterna {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new CivilightEterna operator
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
    ///
    /// atkbuff = 0.08 if self.module == 1 and self.module_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[3]
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale)
    /// dps = hitdmg / 7 * 6
    /// else: dps = res * 0
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

        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        atkbuff = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            0.08
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[3];
            hitdmg =
                ((final_atk * skill_scale - defense) as f64).max((final_atk * skill_scale) as f64);
            dps = hitdmg / 7.0 * 6.0;
        } else {
            dps = res * 0.0;
        }
        return dps;
    }
}

impl std::ops::Deref for CivilightEterna {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for CivilightEterna {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
