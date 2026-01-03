//! DPS calculations for Jessica
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Jessica operator implementation
pub struct Jessica {
    pub unit: OperatorUnit,
}

impl Jessica {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Jessica operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// aspd = max(self.talent1_params)
    /// aspd += 8 if self.module == 2 and self.module_dmg else 1
    /// atkbuff = (self.module_lvl-1) * 0.03 if self.module == 2 and self.module_lvl > 1 else 0
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmg_skill = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// avghit = (hitdmg * self.skill_cost + hitdmg_skill)/(self.skill_cost + 1)
    /// dps = avghit / self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        aspd = self.unit.talent1_parameters.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        aspd += if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage { 8.0 } else { 1.0 };
        atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0 && ((self.unit.module_level as f64) as f64) > 1.0 { (((self.unit.module_level as f64) as f64)-1.0) * 0.03 } else { 0.0 };
        if (self.unit.skill_index as f64) < 2.0 {
        skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[0] } else { 1.0 };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut hitdmg_skill = ((final_atk * skill_scale - defense) as f64).max((final_atk * skill_scale * 0.05) as f64);
        avghit = (hitdmg * (self.unit.skill_cost as f64) + hitdmg_skill)/((self.unit.skill_cost as f64) + 1.0);
        dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff + self.unit.skill_parameters[0]) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Jessica {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Jessica {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
