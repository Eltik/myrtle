//! DPS calculations for Midnight
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Midnight operator implementation
pub struct Midnight {
    pub unit: OperatorUnit,
}

impl Midnight {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Midnight operator
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
    /// atk_scale = 1 if self.trait_dmg else 0.8
    /// crate = self.talent1_params[0] if self.elite > 0 else 0
    /// cdmg = self.talent1_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill) + self.buff_atk_flat
    /// if self.skill == 1:
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg * atk_scale * (1-res/100), final_atk * cdmg * atk_scale * 0.05)
    /// else:
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg * atk_scale - defense, final_atk * cdmg * atk_scale * 0.05)
    /// avghit = crate * critdmg + (1-crate) * hitdmg
    /// dps = avghit / self.atk_interval * self.attack_speed / 100
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

        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut critdmg: f64 = 0.0;

        atk_scale = if self.unit.trait_damage { 1.0 } else { 0.8 };
        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters[0]
        } else {
            0.0
        };
        cdmg = self.unit.talent1_parameters[1];
        final_atk = self.unit.atk
            * (1.0
                + self.unit.buff_atk
                + self.unit.skill_parameters[0] * (self.unit.skill_index as f64))
            + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 1.0 {
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * cdmg * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * cdmg * atk_scale * 0.05) as f64);
        } else {
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * cdmg * atk_scale - defense) as f64)
                .max((final_atk * cdmg * atk_scale * 0.05) as f64);
        }
        avghit = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
        dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        dps
    }
}

impl std::ops::Deref for Midnight {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Midnight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
