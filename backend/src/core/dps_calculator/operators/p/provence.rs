//! DPS calculations for Provence
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Provence operator implementation
pub struct Provence {
    pub unit: OperatorUnit,
}

impl Provence {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Provence operator
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
    /// crate = 0
    /// cdmg = self.talent1_params[2]
    /// if self.elite > 0:
    /// crate = self.talent1_params[1] if self.talent_dmg else self.talent1_params[0]
    /// if self.skill < 2:
    /// skill_scale = 1 + self.skill_params[1] * 5 if self.skill_dmg and self.skill == 1 else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// critdmg = np.fmax(final_atk * skill_scale * cdmg - defense, final_atk * skill_scale * cdmg * 0.05)
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// avghit =  crate * critdmg + (1-crate) * hitdmg
    /// dps = avghit/self.atk_interval * self.attack_speed/100
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

        let mut avghit: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut critdmg: f64 = 0.0;

        let mut crit_rate = 0.0;
        cdmg = self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0);
        if (self.unit.elite as f64) > 0.0 {
            crit_rate = if self.unit.talent_damage {
                self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            };
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale =
                if self.unit.skill_damage && ((self.unit.skill_index as f64) as f64) == 1.0 {
                    1.0 + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0) * 5.0
                } else {
                    1.0
                };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            critdmg = ((final_atk * skill_scale * cdmg - defense) as f64)
                .max((final_atk * skill_scale * cdmg * 0.05) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
        }
        avghit = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
        dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Provence {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Provence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
