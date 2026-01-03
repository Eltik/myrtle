//! DPS calculations for WakabaMutsumi
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// WakabaMutsumi operator implementation
pub struct WakabaMutsumi {
    pub unit: OperatorUnit,
}

impl WakabaMutsumi {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new WakabaMutsumi operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// dps = 3 * hitdmg / (self.atk_interval+self.skill_params[0]) * self.attack_speed / 100
    /// else:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100
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

        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[1];
            hitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps = 3.0 * hitdmg
                / ((self.unit.attack_interval as f64) + self.unit.skill_parameters[0])
                * self.unit.attack_speed
                / 100.0;
        } else {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        dps
    }
}

impl std::ops::Deref for WakabaMutsumi {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for WakabaMutsumi {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
