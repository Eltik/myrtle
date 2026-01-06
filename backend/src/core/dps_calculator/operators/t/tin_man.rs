//! DPS calculations for TinMan
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// TinMan operator implementation
pub struct TinMan {
    pub unit: OperatorUnit,
}

impl TinMan {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new TinMan operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// dmg_bonus = self.talent2_params[0] if self.elite == 2 else 1
    ///
    /// duration = self.skill_params[0]
    /// skill_scale = self.skill_params[2]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_bonus * self.targets
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// if self.skill == 0: return hitdmg/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 1:
    /// sp_cost = self.skill_cost
    /// dps = sp_cost/(sp_cost + 1) * hitdmg / self.atk_interval * (self.attack_speed) / 100
    /// dps += duration * skilldmg / ((sp_cost + 1) * self.atk_interval/self.attack_speed*100)
    /// if self.skill == 2:
    /// dps = hitdmg / self.atk_interval * (self.attack_speed) / 100
    /// dps += skilldmg
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

        let mut skilldmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut dmg_bonus = if ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut duration = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
        skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
            .max((final_atk * skill_scale * 0.05) as f64)
            * dmg_bonus
            * (self.unit.targets as f64);
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        if (self.unit.skill_index as f64) == 0.0 {
            return hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            sp_cost = (self.unit.skill_cost as f64);
            dps = sp_cost / (sp_cost + 1.0) * hitdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed)
                / 100.0;
            dps += duration * skilldmg
                / ((sp_cost + 1.0) * (self.unit.attack_interval as f64) / self.unit.attack_speed
                    * 100.0);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
            dps += skilldmg;
        }
        return dps;
    }
}

impl std::ops::Deref for TinMan {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for TinMan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for TinMan {
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
