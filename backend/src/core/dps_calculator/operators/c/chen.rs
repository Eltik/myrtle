//! DPS calculations for Chen
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Chen operator implementation
pub struct Chen {
    pub unit: OperatorUnit,
}

impl Chen {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[];

    /// Creates a new Chen operator
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
    /// 
    /// dmg = 1.1 if self.module == 1 else 1
    /// atkbuff = self.talent2_params[0] if self.elite == 2 else 0
    /// newdef = np.fmax(0, defense - 70) if self.module == 2 else defense
    /// sp_gain = self.talent1_params[1] / self.talent1_params[0] if self.elite > 0 else 0
    /// if self.module == 1 and self.module_lvl == 3: sp_gain *= 2
    /// 
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05) * 2
    /// if self.skill == 0: dps = hitdmg / self.atk_interval * self.attack_speed/100
    /// if self.skill == 1:
    /// skilldmg = np.fmax(final_atk * skill_scale - newdef, final_atk * skill_scale * 0.05) * dmg
    /// sp_cost = self.skill_cost/(1/(self.atk_interval*self.attack_speed/100) + sp_gain)
    /// avghit = ( int(sp_cost / (1/(self.atk_interval*self.attack_speed/100))) * hitdmg + skilldmg)/(int(sp_cost / (1/(self.atk_interval*self.attack_speed/100)))+1)
    /// dps = avghit / self.atk_interval * self.attack_speed/100
    /// 
    /// if self.skill == 2:
    /// hitdmgphys = np.fmax(final_atk * skill_scale - newdef, final_atk * skill_scale * 0.05) * dmg
    /// hitdmgarts = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg
    /// skilldmg = hitdmgphys + hitdmgarts
    /// sp_cost = self.skill_cost/(1/(self.atk_interval*self.attack_speed/100) + sp_gain)
    /// dps = hitdmg / self.atk_interval * self.attack_speed/100 + skilldmg/sp_cost * min(self.targets, self.skill_params[1])
    /// 
    /// if self.skill == 3:
    /// hitdmg = np.fmax(final_atk * skill_scale - newdef, final_atk * skill_scale * 0.05) * dmg
    /// dps = 10 * hitdmg
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        let mut dmg = if ((self.unit.module_index as f64) as f64) == 1.0 { 1.1 } else { 1.0 };
        atkbuff = if ((self.unit.elite as f64) as f64) == 2.0 { self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        let mut newdef = if ((self.unit.module_index as f64) as f64) == 2.0 { ((0) as f64).max((defense - 70.0) as f64) } else { defense };
        let mut sp_gain = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0) / self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        if (self.unit.module_index as f64) == 1.0 && (self.unit.module_level as f64) == 3.0 { sp_gain *= 2.0; }
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64) * 2.0;
        if (self.unit.skill_index as f64) == 0.0 { dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0; }
        if (self.unit.skill_index as f64) == 1.0 {
        skilldmg = ((final_atk * skill_scale - newdef) as f64).max((final_atk * skill_scale * 0.05) as f64) * dmg;
        sp_cost = (self.unit.skill_cost as f64)/(1.0 /((self.unit.attack_interval as f64)*self.unit.attack_speed/ 100.0) + sp_gain);
        avghit = ( ((sp_cost / (1.0 /((self.unit.attack_interval as f64)*self.unit.attack_speed/ 100.0))) as f64).trunc() * hitdmg + skilldmg)/(((sp_cost / (1.0 /((self.unit.attack_interval as f64)*self.unit.attack_speed/ 100.0))) as f64).trunc()+1.0);
        dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        let mut hitdmgphys = ((final_atk * skill_scale - newdef) as f64).max((final_atk * skill_scale * 0.05) as f64) * dmg;
        hitdmgarts = ((final_atk * skill_scale * (1.0 -res/ 100.0)) as f64).max((final_atk * skill_scale * 0.05) as f64) * dmg;
        skilldmg = hitdmgphys + hitdmgarts;
        sp_cost = (self.unit.skill_cost as f64)/(1.0 /((self.unit.attack_interval as f64)*self.unit.attack_speed/ 100.0) + sp_gain);
        dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 + skilldmg/sp_cost * (((self.unit.targets as f64)) as f64).min((self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
        hitdmg = ((final_atk * skill_scale - newdef) as f64).max((final_atk * skill_scale * 0.05) as f64) * dmg;
        dps = 10.0 * hitdmg;
        }
        return dps;
    }
}

impl std::ops::Deref for Chen {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Chen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Chen {
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
