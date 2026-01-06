//! DPS calculations for Saga
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Saga operator implementation
pub struct Saga {
    pub unit: OperatorUnit,
}

impl Saga {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("module", "blocking", false, &[], &[1], 0, 0),
        ("talent", "vsLowHp", false, &[], &[1], 0, 2),
    ];

    /// Creates a new Saga operator
    #[allow(unused_parens)]
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
    /// atkbuff = 0.08 if self.module_dmg and self.module == 1 else 0
    /// dmg = self.module_lvl * 0.05 if self.module == 1 and self.module_lvl > 1 else 0
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * min(self.targets,6)
    /// sp_cost = self.skill_cost/(1 + self.sp_boost) + 1.2
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + skilldmg/sp_cost
    /// if self.skill == 3:
    /// self.atk_interval = 1.55
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk  - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * min(self.targets,2)
    /// return dps * (1+dmg)
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

        let mut sp_cost: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        atkbuff = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 1.0 {
            0.08
        } else {
            0.0
        };
        let mut dmg = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            ((self.unit.module_level as f64) as f64) * 0.05
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / atk_interval * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((6) as f64);
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            dps = hitdmg / atk_interval * self.unit.attack_speed / 100.0 + skilldmg / sp_cost;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = 1.55;
            final_atk = self.unit.atk
                * (1.0
                    + atkbuff
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / atk_interval * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps * (1.0 + dmg);
    }
}

impl std::ops::Deref for Saga {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Saga {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Saga {
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
