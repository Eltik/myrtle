//! DPS calculations for Eyjafjalla
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Eyjafjalla operator implementation
pub struct Eyjafjalla {
    pub unit: OperatorUnit,
}

impl Eyjafjalla {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Eyjafjalla operator
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
    /// atkbuff = self.talent1_params[0] if self.elite > 0 else 0
    /// resignore = 10 if self.module == 1 else 0
    /// newres = np.fmax(0, res - resignore)
    /// aspd = 0
    /// if self.module == 2 and self.module_lvl == 3:
    /// if self.talent_dmg: aspd = 16
    /// else: aspd = 6
    /// 
    /// if self.skill < 2:
    /// aspd += self.skill_params[0] if self.skill == 1 else 0
    /// if self.skill_dmg and self.skill == 1: atkbuff += self.skill_params[2]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-newres/100), final_atk * 0.05)
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[2]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// newres2 = np.fmax(0, res*(1+self.skill_params[5])-resignore)
    /// hitdmg = np.fmax(final_atk  * (1-newres2/100), final_atk * 0.05)
    /// if not self.skill_dmg: hitdmg = np.fmax(final_atk  * (1-newres/100), final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale * (1-newres2/100), final_atk* atk_scale * 0.05)
    /// aoeskilldmg = np.fmax(0.5 * final_atk * atk_scale * (1-newres/100), 0.5 * final_atk* atk_scale * 0.05)
    /// extra_boost = 1/(self.atk_interval)*(self.attack_speed+aspd)/100 if self.module == 2 and self.module_dmg else 0
    /// sp_cost = self.skill_cost/(1+self.sp_boost + extra_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/((self.attack_speed+aspd)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg + (self.targets - 1) * aoeskilldmg
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[3] > 1:
    /// avghit = (skilldmg + (self.targets - 1) * aoeskilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + (self.targets - 1) * aoeskilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * (self.attack_speed+aspd)/100
    /// 
    /// if self.skill == 3:
    /// self.atk_interval = 0.5
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-newres/100), final_atk * 0.05)
    /// maxtargets = self.skill_params[2]
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100 * min(self.targets, maxtargets)
    /// 
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;

        atkbuff = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters[0] } else { 0.0 };
        let mut resignore = if ((self.unit.module_index as f64) as f64) == 1.0 { 10.0 } else { 0.0 };
        newres = ((0) as f64).max((res - resignore) as f64);
        aspd = 0.0;
        if (self.unit.module_index as f64) == 2.0 && (self.unit.module_level as f64) == 3.0 {
        if self.unit.talent_damage { aspd = 16.0; }
        else { aspd = 6.0; }
        }
        if (self.unit.skill_index as f64) < 2.0 {
        aspd += if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[0] } else { 0.0 };
        // UNTRANSLATED: if self.skill_dmg and self.skill == 1: atkbuff += self.skill_params[2]
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk *(1.0 -newres/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmgarts/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        atk_scale = self.unit.skill_parameters[2];
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut newres2 = ((0) as f64).max((res*(1.0 +self.unit.skill_parameters[5])-resignore) as f64);
        hitdmg = ((final_atk  * (1.0 -newres2/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        if !self.unit.skill_damage { hitdmg = ((final_atk  * (1.0 -newres/ 100.0)) as f64).max((final_atk * 0.05) as f64); }
        skilldmg = ((final_atk * atk_scale * (1.0 -newres2/ 100.0)) as f64).max((final_atk* atk_scale * 0.05) as f64);
        let mut aoeskilldmg = ((0.5 * final_atk * atk_scale * (1.0 -newres/ 100.0)) as f64).max((0.5 * final_atk* atk_scale * 0.05) as f64);
        let mut extra_boost = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage { 1.0 /((self.unit.attack_interval as f64))*(self.unit.attack_speed+aspd)/ 100.0 } else { 0.0 };
        sp_cost = (self.unit.skill_cost as f64)/(1.0 +(self.unit.sp_boost as f64) + extra_boost) + 1.2;
        let mut atkcycle = (self.unit.attack_interval as f64)/((self.unit.attack_speed+aspd)/ 100.0);
        let mut atks_per_skillactivation = sp_cost / atkcycle;
        avghit = skilldmg + ((self.unit.targets as f64) - 1.0) * aoeskilldmg;
        if atks_per_skillactivation > 1.0 {
        if self.unit.skill_parameters[3] > 1.0 {
        avghit = (skilldmg + ((self.unit.targets as f64) - 1.0) * aoeskilldmg + (atks_per_skillactivation - 1.0) * hitdmg) / atks_per_skillactivation;
        } else {
        avghit = (skilldmg + ((self.unit.targets as f64) - 1.0) * aoeskilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg) / (((atks_per_skillactivation) as f64).trunc()+1.0);
        }
        }
        dps = avghit/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
        // UNTRANSLATED: self.atk_interval = 0.5
        atkbuff += self.unit.skill_parameters[0];
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk *(1.0 -newres/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut maxtargets = self.unit.skill_parameters[2];
        dps = hitdmgarts/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0 * (((self.unit.targets as f64)) as f64).min((maxtargets) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Eyjafjalla {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Eyjafjalla {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
