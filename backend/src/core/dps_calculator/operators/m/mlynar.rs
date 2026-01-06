//! DPS calculations for Mlynar
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Mlynar operator implementation
pub struct Mlynar {
    pub unit: OperatorUnit,
}

impl Mlynar {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[
        ("trait", "-10stacks", true, &[], &[], 0, 0),
        ("talent", "3+Nearby", false, &[], &[], 1, 0),
    ];

    /// Creates a new Mlynar operator
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
    ///
    /// atkbuff = 0
    /// atk_scale = 1
    /// if self.elite > 0: atk_scale = self.talent1_params[2] if self.talent_dmg or self.targets > 2 else self.talent1_params[0]
    /// stacks = 40
    /// if not self.trait_dmg: stacks -= 10
    /// atkbuff += stacks * 0.05
    /// if self.skill == 0: dps = res * 0
    /// if self.skill == 1:
    /// atk_scale *= self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// finaldmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = finaldmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// self.atk_interval = 1.5
    /// atk_scale *= self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// finaldmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * 2
    /// dps = finaldmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 3:
    /// atkbuff += stacks * 0.05
    /// atk_scale *= self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// truedmg = final_atk * self.skill_params[1] * np.fmax(1,-defense) #this defense part has to be included
    /// finaldmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = (finaldmg + truedmg)/self.atk_interval * self.attack_speed/100
    /// dps = dps * min(self.targets, 5)
    /// if self.hits > 0 and self.elite == 2:
    /// truescaling = self.talent2_params[1]
    /// dps += final_atk * truescaling * self.hits * np.fmax(1,-defense) #this defense part has to be included
    ///
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

        let mut final_atk: f64 = 0.0;
        let mut finaldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atkbuff = 0.0;
        atk_scale = 1.0;
        if (self.unit.elite as f64) > 0.0 {
            atk_scale = if self.unit.talent_damage || (self.unit.targets as f64) > 2.0 {
                self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            };
        }
        let mut stacks = 40.0;
        if !self.unit.trait_damage {
            stacks -= 10.0;
        }
        atkbuff += stacks * 0.05;
        if (self.unit.skill_index as f64) == 0.0 {
            dps = res * 0.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            atk_scale *= self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            finaldmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = finaldmg / atk_interval * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = 1.5;
            atk_scale *= self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            finaldmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * 2.0;
            dps = finaldmg / atk_interval * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += stacks * 0.05;
            atk_scale *= self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut truedmg = final_atk
                * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                * ((1) as f64).max((-defense) as f64);
            finaldmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = (finaldmg + truedmg) / atk_interval * self.unit.attack_speed / 100.0;
            dps = dps * ((self.unit.targets as f64) as f64).min((5) as f64);
        }
        if 0.0 /* self.hits - defaults to 0 */ > 0.0 && (self.unit.elite as f64) == 2.0 {
            let mut truescaling = self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0);
            dps += final_atk * truescaling * 0.0 /* self.hits - defaults to 0 */ * ((1) as f64).max((-defense) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Mlynar {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Mlynar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Mlynar {
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
