//! DPS calculations for Kroos
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Kroos operator implementation
pub struct Kroos {
    pub unit: OperatorUnit,
}

impl Kroos {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Kroos operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// crate = 0 if self.elite == 0 else self.talent1_params[0]
    /// cdmg = self.talent1_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitcrit = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * 2
    /// skillcrit =  np.fmax(final_atk * skill_scale * cdmg - defense, final_atk * skill_scale * cdmg * 0.05) * 2
    /// avghit = crate * hitcrit + (1-crate) * hitdmg
    /// avgskill = crate * skillcrit + (1-crate) * skilldmg
    /// avgdmg = (avghit * self.skill_cost + avgskill) / (self.skill_cost+1)
    /// dps = avgdmg/self.atk_interval * self.attack_speed/100
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

        let mut crit_rate = if ((self.unit.elite as f64) as f64) == 0.0 {
            0.0
        } else {
            self.unit.talent1_parameters[0]
        };
        let mut cdmg = self.unit.talent1_parameters[1];
        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut skill_scale = self.unit.skill_parameters[0];
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut hitcrit =
            ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
        let mut skilldmg = ((final_atk * skill_scale - defense) as f64)
            .max((final_atk * skill_scale * 0.05) as f64)
            * 2.0;
        let mut skillcrit = ((final_atk * skill_scale * cdmg - defense) as f64)
            .max((final_atk * skill_scale * cdmg * 0.05) as f64)
            * 2.0;
        let mut avghit = crit_rate * hitcrit + (1.0 - crit_rate) * hitdmg;
        let mut avgskill = crit_rate * skillcrit + (1.0 - crit_rate) * skilldmg;
        let mut avgdmg = (avghit * (self.unit.skill_cost as f64) + avgskill)
            / ((self.unit.skill_cost as f64) + 1.0);
        
        avgdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
    }
}

impl std::ops::Deref for Kroos {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Kroos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
