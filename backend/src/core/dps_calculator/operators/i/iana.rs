//! DPS calculations for Iana
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Iana operator implementation
pub struct Iana {
    pub unit: OperatorUnit,
}

impl Iana {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Iana operator
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
    /// atkbuff = 0.15 if self.module == 1 else 0
    /// fragile = self.talent1_params[2] - 1
    /// fragile = max(fragile, self.buff_fragile)
    /// aspd = self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * (1+fragile)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 /(1+self.buff_fragile)
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

        let mut atkbuff = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.15
        } else {
            0.0
        };
        let mut fragile = self.unit.talent1_parameters[2] - 1.0;
        fragile = ((fragile) as f64).max((self.unit.buff_fragile) as f64);
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters[0]
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * (1.0 + fragile);
        
        hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0
            / (1.0 + self.unit.buff_fragile)
    }
}

impl std::ops::Deref for Iana {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Iana {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
