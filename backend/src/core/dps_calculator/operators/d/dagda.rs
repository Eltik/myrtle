//! DPS calculations for Dagda
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Dagda operator implementation
pub struct Dagda {
    pub unit: OperatorUnit,
}

impl Dagda {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Dagda operator
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
    /// aspd = 10 if self.module == 1 and self.module_dmg else 0
    /// crate = 0.3
    /// cdmg = self.talent1_params[2] if self.talent_dmg else self.talent1_params[1]
    /// if self.elite == 0: cdmg = 1
    /// if self.skill == 2: crate = self.skill_params[1]
    /// hits = 2 if self.skill == 2 else 1
    /// atkbuff = self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = hits * avgdmg/self.atk_interval * (self.attack_speed+aspd)/100
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

        let mut aspd = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage
        {
            10.0
        } else {
            0.0
        };
        let mut crit_rate = 0.3;
        let mut cdmg = if self.unit.talent_damage {
            self.unit.talent1_parameters[2]
        } else {
            self.unit.talent1_parameters[1]
        };
        if (self.unit.elite as f64) == 0.0 {
            cdmg = 1.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            crit_rate = self.unit.skill_parameters[1];
        }
        let mut hits = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            2.0
        } else {
            1.0
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters[0]
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut critdmg =
            ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
        let mut avgdmg = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
        
        hits * avgdmg / (self.unit.attack_interval as f64)
            * (self.unit.attack_speed + aspd)
            / 100.0
    }
}

impl std::ops::Deref for Dagda {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Dagda {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
