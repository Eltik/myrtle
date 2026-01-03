//! DPS calculations for Snegurochka
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Snegurochka operator implementation
pub struct Snegurochka {
    pub unit: OperatorUnit,
}

impl Snegurochka {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Snegurochka operator
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
    /// aspd = self.talent1_params[0] if self.elite > 0 and self.talent_dmg else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// 
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[1] if self.skill == 1 else 1
    /// hitdmg_skill = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// avghit = (hitdmg * self.skill_cost + hitdmg_skill)/(self.skill_cost + 1)
    /// dps = avghit / self.atk_interval * (self.attack_speed+aspd)/100
    /// 
    /// if self.skill == 2:
    /// aspd += self.skill_params[0]
    /// dps = hitdmg / self.atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        aspd = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage { self.unit.talent1_parameters[0] } else { 0.0 };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        if (self.unit.skill_index as f64) < 2.0 {
        skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[1] } else { 1.0 };
        let mut hitdmg_skill = ((final_atk * skill_scale - defense) as f64).max((final_atk * skill_scale * 0.05) as f64);
        avghit = (hitdmg * (self.unit.skill_cost as f64) + hitdmg_skill)/((self.unit.skill_cost as f64) + 1.0);
        dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        aspd += self.unit.skill_parameters[0];
        dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Snegurochka {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Snegurochka {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
