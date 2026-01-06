//! DPS calculations for Hoshiguma
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Hoshiguma operator implementation
pub struct Hoshiguma {
    pub unit: OperatorUnit,
}

impl Hoshiguma {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("module", "afterDodge", false, &[], &[2], 0, 2)];

    /// Creates a new Hoshiguma operator
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
    /// atkbuff = self.talent1_params[1] if self.module == 2 and self.module_lvl > 1 and self.module_dmg else 0
    /// targets = self.targets if self.skill == 3 else 1
    /// if self.skill == 3: atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * targets
    /// if self.skill == 2 and self.hits > 0:
    /// skill_scale = self.skill_params[0]
    /// reflectdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// dps += reflectdmg * self.hits
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

        let mut hitdmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && self.unit.module_damage
        {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut targets = if ((self.unit.skill_index as f64) as f64) == 3.0 {
            (self.unit.targets as f64)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps =
            hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0 * targets;
        if (self.unit.skill_index as f64) == 2.0 && 0.0 /* self.hits - defaults to 0 */ > 0.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            let mut reflectdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps += reflectdmg * 0.0 /* self.hits - defaults to 0 */;
        }
        return dps;
    }
}

impl std::ops::Deref for Hoshiguma {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Hoshiguma {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Hoshiguma {
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
