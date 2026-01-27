//! DPS calculations for Narantuya
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Narantuya operator implementation
pub struct Narantuya {
    pub unit: OperatorUnit,
}

impl Narantuya {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("trait", "maxRange", true, &[], &[], 0, 0),
        ("talent", "maxSteal", false, &[], &[], 1, 0),
    ];

    /// Creates a new Narantuya operator
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
    /// stealbuff = self.talent1_params[1] if self.elite > 0 and self.talent_dmg else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat + stealbuff
    /// atk_scale = 1.1 if self.module == 1 and self.trait_dmg else 1
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// interval = self.atk_interval/self.attack_speed*100 if self.trait_dmg else 2.1
    /// dps = hitdmg / interval
    /// if self.targets > 1: dps *= 3
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[2]
    /// return_scale = self.skill_params[3]
    /// hitdmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// returndmg = np.fmax(final_atk * return_scale * atk_scale - defense, final_atk * return_scale * atk_scale * 0.05) * self.targets
    /// interval = 1.15 if self.trait_dmg else 2
    /// dps = (hitdmg+returndmg) / interval
    /// if self.skill in [0,3]:
    /// skill_scale = self.skill_params[0] if self.skill == 3 else 1
    /// aoe_scale = self.skill_params[1] if self.skill == 3 else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat + stealbuff
    /// hitdmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05) * max(self.skill,1)
    /// aoedmg = np.fmax(final_atk * aoe_scale * atk_scale - defense, final_atk * aoe_scale * atk_scale * 0.05)
    /// if not self.trait_dmg: aoedmg = 0
    /// interval = 20/13.6 if not self.trait_dmg else (self.atk_interval/(self.attack_speed/100))
    /// dps = hitdmg/interval + min(self.targets,3) * aoedmg/interval
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

        let mut atk_scale: f64 = 0.0;
        let mut interval: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;

        let mut stealbuff = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat + stealbuff;
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.trait_damage {
            1.1
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            interval = if self.unit.trait_damage {
                (self.unit.attack_interval as f64) / self.unit.attack_speed * 100.0
            } else {
                2.1
            };
            dps = hitdmg / interval;
            if (self.unit.targets as f64) > 1.0 {
                dps *= 3.0;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            let mut return_scale = self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            let mut returndmg = ((final_atk * return_scale * atk_scale - defense) as f64)
                .max((final_atk * return_scale * atk_scale * 0.05) as f64)
                * (self.unit.targets as f64);
            interval = if self.unit.trait_damage { 1.15 } else { 2.0 };
            dps = (hitdmg + returndmg) / interval;
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 3.0 {
                self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            let mut aoe_scale = if ((self.unit.skill_index as f64) as f64) == 3.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat + stealbuff;
            hitdmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64)
                * ((self.unit.skill_index as f64) as f64).max((1) as f64);
            let mut aoedmg = ((final_atk * aoe_scale * atk_scale - defense) as f64)
                .max((final_atk * aoe_scale * atk_scale * 0.05) as f64);
            if !self.unit.trait_damage {
                aoedmg = 0.0;
            }
            interval = if !self.unit.trait_damage {
                20.0 / 13.6
            } else {
                ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
            };
            dps = hitdmg / interval
                + ((self.unit.targets as f64) as f64).min((3) as f64) * aoedmg / interval;
        }
        return dps;
    }
}

impl std::ops::Deref for Narantuya {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Narantuya {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Narantuya {
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
