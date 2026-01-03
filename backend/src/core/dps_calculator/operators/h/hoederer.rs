//! DPS calculations for Hoederer
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Hoederer operator implementation
pub struct Hoederer {
    pub unit: OperatorUnit,
}

impl Hoederer {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Hoederer operator
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
    /// atk_scale = 1
    /// if self.elite > 0:
    /// atk_scale = max(self.talent1_params) if self.talent_dmg else min(self.talent1_params)
    /// dmg_bonus = 1
    /// if self.module == 1:
    /// if self.module_lvl == 2: dmg_bonus = 1.06
    /// if self.module_lvl == 3: dmg_bonus = 1.1
    /// 
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1)
    /// dps = avgphys/self.atk_interval * self.attack_speed/100 * min(self.targets,2) * dmg_bonus
    /// if self.skill == 2:
    /// maxtargets = 3 if self.skill_dmg else 2
    /// if self.skill_dmg: self.atk_interval = 3
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * min(self.targets,maxtargets) * dmg_bonus
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// if not self.talent_dmg and self.skill_dmg:
    /// stun_duration = self.skill_params[4]
    /// atk_cycle = self.atk_interval / self.attack_speed * 100
    /// counting_hits = int(stun_duration/atk_cycle) + 1
    /// chance_to_attack_stunned = 1 - 0.75 ** counting_hits
    /// atk_scale = max(self.talent1_params)
    /// hitdmg2 = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// hitdmg = chance_to_attack_stunned * hitdmg2 + (1-chance_to_attack_stunned)*hitdmg
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * dmg_bonus  + 200
    /// dps = dps * min(2, self.targets)
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut skill_scale: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut chance_to_attack_stunned: f64 = 0.0;
        let mut stun_duration: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut counting_hits: f64 = 0.0;
        let mut atk_cycle: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg2: f64 = 0.0;

        atk_scale = 1.0;
        if (self.unit.elite as f64) > 0.0 {
        atk_scale = if self.unit.talent_damage { self.unit.talent1_parameters.iter().cloned().fold(f64::NEG_INFINITY, f64::max) } else { self.unit.talent1_parameters.iter().cloned().fold(f64::INFINITY, f64::min) };
        }
        let mut dmg_bonus = 1.0;
        if (self.unit.module_index as f64) == 1.0 {
        if (self.unit.module_level as f64) == 2.0 { dmg_bonus = 1.06; }
        if (self.unit.module_level as f64) == 3.0 { dmg_bonus = 1.1; }
        }
        if (self.unit.skill_index as f64) < 2.0 {
        skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[0] } else { 1.0 };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut skillhitdmg = ((final_atk * atk_scale * skill_scale - defense) as f64).max((final_atk * atk_scale * skill_scale * 0.05) as f64);
        sp_cost = (self.unit.skill_cost as f64);
        avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1.0);
        dps = avgphys/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 * (((self.unit.targets as f64)) as f64).min((2) as f64) * dmg_bonus;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        let mut maxtargets = if self.unit.skill_damage { 3.0 } else { 2.0 };
        // UNTRANSLATED: if self.skill_dmg: self.atk_interval = 3
        atkbuff = self.unit.skill_parameters[0];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 * (((self.unit.targets as f64)) as f64).min((maxtargets) as f64) * dmg_bonus;
        }
        if (self.unit.skill_index as f64) == 3.0 {
        atkbuff = self.unit.skill_parameters[1];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        if !self.unit.talent_damage && self.unit.skill_damage {
        stun_duration = self.unit.skill_parameters[4];
        atk_cycle = (self.unit.attack_interval as f64) / self.unit.attack_speed * 100.0;
        counting_hits = ((stun_duration/atk_cycle) as f64).trunc() + 1.0;
        chance_to_attack_stunned = 1.0 - (0.75 as f64).powf(counting_hits as f64);
        atk_scale = self.unit.talent1_parameters.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        hitdmg2 = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        hitdmg = chance_to_attack_stunned * hitdmg2 + (1.0 -chance_to_attack_stunned)*hitdmg;
        }
        dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 * dmg_bonus  + 200.0;
        dps = dps * ((2) as f64).min(((self.unit.targets as f64)) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Hoederer {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Hoederer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
