//! DPS calculations for Matoimaru
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Matoimaru operator implementation
pub struct Matoimaru {
    pub unit: OperatorUnit,
}

impl Matoimaru {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Matoimaru operator
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
    /// aspd = 30 if self.module_dmg and self.module == 2 else 0
    /// atkbuff = self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut aspd = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 2.0
        {
            30.0
        } else {
            0.0
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters[0]
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        
        hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0
    }
}

impl std::ops::Deref for Matoimaru {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Matoimaru {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
