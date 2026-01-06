//! DPS calculations for MrNothing
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// MrNothing operator implementation
pub struct MrNothing {
    pub unit: OperatorUnit,
}

impl MrNothing {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "idealTalentUsage", true, &[], &[], 1, 0),
        ("skill", "Apsd+Skill", false, &[2], &[], 0, 0),
    ];

    /// Creates a new MrNothing operator
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
    /// atkbuff = self.skill_params[0] if self.skill == 2 else 0
    /// aspd = self.skill_params[3] if self.skill == 2 and self.skill_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk * self.talent1_params[1] - defense, final_atk * self.talent1_params[1] * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 if self.talent_dmg else hitdmg2 / self.talent1_params[0]
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

        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 && self.unit.skill_damage {
            self.unit.skill_parameters.get(3).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut hitdmg2 = ((final_atk * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
            - defense) as f64)
            .max(
                (final_atk * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0) * 0.05)
                    as f64,
            );
        let mut dps = if self.unit.talent_damage {
            hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0
        } else {
            hitdmg2 / self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        };
        return dps;
    }
}

impl std::ops::Deref for MrNothing {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for MrNothing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for MrNothing {
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
