//! DPS calculations for Marcille
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Marcille operator implementation
pub struct Marcille {
    pub unit: OperatorUnit,
}

impl Marcille {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "noMana", true, &[], &[], 0, 0),
        ("talent2", "FullSquad", false, &[], &[], 2, 0),
        ("skill", "2ndActivation", false, &[2], &[], 0, 0),
    ];

    /// Creates a new Marcille operator
    #[allow(unused_parens)]
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
    /// dps = 0
    /// atkbuff = self.talent1_params[5] if self.talent_dmg else 0
    /// aspd = self.talent2_params[1] if self.talent2_dmg else 0
    /// if self.skill < 2:
    /// atkbuff += self.skill_params[3] if self.skill == 1 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * self.targets
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[3]
    /// aspd += self.skill_params[5] if self.skill_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * self.targets
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[6]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale *  0.05)
    /// dps = hitdmg * self.targets
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        dps = 0.0;
        atkbuff = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(5).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        aspd = if self.unit.talent2_damage {
            self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff += if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(3).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            aspd += if self.unit.skill_damage {
                self.unit.skill_parameters.get(5).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.get(6).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps = hitdmg * (self.unit.targets as f64);
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    /// if self.skill == 3: self.skill_duration = 10
    /// return super().total_dmg(defense, res)
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut skill_duration: f64 = 0.0;
        if (self.unit.skill_index as f64) == 3.0 {
            skill_duration = 10.0;
        }
        // TODO: return super().total_dmg(defense, res) - requires manual implementation
        0.0 // placeholder
    }
}

impl std::ops::Deref for Marcille {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Marcille {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Marcille {
    fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        Self::skill_dps(self, enemy)
    }

    fn unit(&self) -> &OperatorUnit {
        &self.unit
    }

    fn unit_mut(&mut self) -> &mut OperatorUnit {
        &mut self.unit
    }
}
