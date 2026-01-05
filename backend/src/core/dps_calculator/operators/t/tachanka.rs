//! DPS calculations for Tachanka
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Tachanka operator implementation
pub struct Tachanka {
    pub unit: OperatorUnit,
}

impl Tachanka {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Tachanka operator
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
    /// dmg_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.skill == 0:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = 2 * hitdmg / self.atk_interval * self.attack_speed /100
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[1]
    /// newdef = np.fmax(0,defense - self.skill_params[2]) if self.skill_dmg else defense
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_scale
    /// dps = 2 * hitdmg / self.atk_interval * self.attack_speed/100 + hitdmgarts * self.targets
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval * 0.15
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * self.skill_params[2] - defense, final_atk * self.skill_params[2] * 0.05)
    /// avghit  = critdmg * self.skill_params[1] + hitdmg * (1 -self.skill_params[1])
    /// dps = 2 * avghit / atk_interval * self.attack_speed/100 * dmg_scale
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
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut hitdmgarts: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut critdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut dps: f64 = 0.0;

        let mut dmg_scale =
            if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
                1.1
            } else {
                1.0
            };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 0.0 {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps =
                2.0 * hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            let mut newdef = if self.unit.skill_damage {
                ((0) as f64).max(
                    (defense - self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)) as f64,
                )
            } else {
                defense
            };
            hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_scale;
            dps = 2.0 * hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                + hitdmgarts * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = (self.unit.attack_interval as f64) * 0.15;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk * self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
                - defense) as f64)
                .max(
                    (final_atk * self.unit.skill_parameters.get(2).copied().unwrap_or(0.0) * 0.05)
                        as f64,
                );
            avghit = critdmg * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                + hitdmg * (1.0 - self.unit.skill_parameters.get(1).copied().unwrap_or(0.0));
            dps = 2.0 * avghit / atk_interval * self.unit.attack_speed / 100.0 * dmg_scale;
        }
        return dps;
    }
}

impl std::ops::Deref for Tachanka {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Tachanka {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
