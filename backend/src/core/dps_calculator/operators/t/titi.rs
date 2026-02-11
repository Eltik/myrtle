//! DPS calculations for Titi
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Titi operator implementation
pub struct Titi {
    pub unit: OperatorUnit,
}

impl Titi {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("skill", "vsSleep", false, &[], &[], 1, 0),
        ("talent", "vsMoving", false, &[], &[], 1, 0),
    ];

    /// Creates a new Titi operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// atk_scale = 1 + self.talent1_params[1] if self.talent_dmg and self.elite > 0 else 1
    /// aspd = self.talent2_params[0] if self.talent2_dmg and self.elite > 1 else 0
    /// final_atk = 0
    /// dps = res * 0
    /// sleep_scale = self.talent1_params[0] if self.elite > 0 and self.skill_dmg else 0
    /// if self.skill < 2:
    /// atkbuff = self.skill * self.skill_params[2]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// atk_scale = 1 + self.talent1_params[1] * self.skill_params[0] if self.talent_dmg and self.elite > 0 else 1
    /// sleep_scale *= self.skill_params[0]
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[0]
    /// skill_scale = self.skill_params[5]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// hitdmgnuke = np.fmax(final_atk * skill_scale * atk_scale * (1-res/100), final_atk * skill_scale * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * min(self.targets,2) + hitdmgnuke/5 * self.targets
    /// if self.skill_dmg and self.elite > 0:
    /// dps += np.fmax(final_atk * sleep_scale * atk_scale * (1-res/100), final_atk * sleep_scale * atk_scale * 0.05) * self.targets
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

        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        atk_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            1.0 + self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        aspd = if self.unit.talent2_damage && ((self.unit.elite as f64) as f64) > 1.0 {
            self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = 0.0;
        dps = res * 0.0;
        let mut sleep_scale = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.skill_damage {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff = (self.unit.skill_index as f64)
                * self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            atk_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
                1.0 + self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
                    * self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            sleep_scale *= self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            skill_scale = self.unit.skill_parameters.get(5).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut hitdmgnuke = ((final_atk * skill_scale * atk_scale * (1.0 - res / 100.0))
                as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64)
                + hitdmgnuke / 5.0 * (self.unit.targets as f64);
        }
        if self.unit.skill_damage && (self.unit.elite as f64) > 0.0 {
            dps += ((final_atk * sleep_scale * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * sleep_scale * atk_scale * 0.05) as f64)
                * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Titi {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Titi {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Titi {
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
