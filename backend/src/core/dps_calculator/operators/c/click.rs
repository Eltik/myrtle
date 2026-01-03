//! DPS calculations for Click
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Click operator implementation
pub struct Click {
    pub unit: OperatorUnit,
}

impl Click {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Click operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// aspd = self.talent1_params[0]
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// drone_dmg = 1.2 if self.module == 2 else 1.1
    /// if not self.trait_dmg: drone_dmg = 0.2
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// dmgperinterval = final_atk + drone_dmg * final_atk
    /// hitdmgarts = np.fmax(dmgperinterval *(1-res/100), dmgperinterval * 0.05)
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed + aspd) / 100
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

        let mut aspd = self.unit.talent1_parameters[0];
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters[0]
        } else {
            0.0
        };
        let mut drone_dmg = if ((self.unit.module_index as f64) as f64) == 2.0 {
            1.2
        } else {
            1.1
        };
        if !self.unit.trait_damage {
            drone_dmg = 0.2;
        }
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut dmgperinterval = final_atk + drone_dmg * final_atk;
        let mut hitdmgarts =
            ((dmgperinterval * (1.0 - res / 100.0)) as f64).max((dmgperinterval * 0.05) as f64);
        let mut dps = hitdmgarts / (self.unit.attack_interval as f64)
            * (self.unit.attack_speed + aspd)
            / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Click {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Click {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
