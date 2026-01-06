//! DPS calculations for Astesia
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Astesia operator implementation
pub struct Astesia {
    pub unit: OperatorUnit,
}

impl Astesia {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "maxStacks", false, &[], &[], 0, 0),
        ("module", "blocking", false, &[], &[2], 0, 0),
    ];

    /// Creates a new Astesia operator
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
    ///
    /// dmg = 1.1 if self.module == 2 and self.module_dmg else 1
    /// aspd = self.talent1_params[0] * self.talent1_params[2] if self.talent_dmg else 0
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd)/100 * dmg
    /// if self.skill == 2: dps *= min(self.targets, 2)
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

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut dmg = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage
        {
            1.1
        } else {
            1.0
        };
        let mut aspd = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
                * self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0
            * dmg;
        if (self.unit.skill_index as f64) == 2.0 {
            dps *= ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Astesia {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Astesia {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Astesia {
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
