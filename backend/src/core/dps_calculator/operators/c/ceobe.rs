//! DPS calculations for Ceobe
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Ceobe operator implementation
pub struct Ceobe {
    pub unit: OperatorUnit,
}

impl Ceobe {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("talent", "maxTalent1", false, &[], &[], 0, 0), ("talent2", "adjacentAlly", true, &[], &[], 2, 0), ("module", "vsElite", false, &[], &[], 0, 0)];

    /// Creates a new Ceobe operator
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
    /// newres= np.fmax(0, res-10) if self.module == 1 else res
    /// bonus_arts_scaling = self.talent1_params[0] if self.elite > 0 else 0
    /// if self.module == 1 and self.module_lvl > 1 and self.talent_dmg: bonus_arts_scaling = self.talent1_params[2]
    /// atkbuff = self.talent2_params[0] if self.elite == 2 and self.talent2_dmg else 0
    /// aspd = self.talent2_params[1] if self.elite == 2 and self.talent2_dmg else 0
    /// 
    /// if self.skill < 2:
    /// sp_cost = self.skill_cost
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-newres/100), final_atk * 0.05)
    /// skilldmgarts = np.fmax(final_atk * skill_scale *(1-newres/100), final_atk * skill_scale * 0.05)
    /// if self.skill == 0: skilldmgarts = hitdmgarts
    /// defbonusdmg = np.fmax(defense * bonus_arts_scaling *(1-newres/100), defense * bonus_arts_scaling * 0.05)
    /// atkcycle = self.atk_interval/(self.attack_speed+aspd)*100
    /// if self.module == 2 and self.module_dmg:
    /// sp_cost = sp_cost / (1 + 1/atkcycle + self.sp_boost) + 1.2 #bonus sp recovery vs elite mobs + sp lockout
    /// else:
    /// sp_cost = sp_cost /(1 + self.sp_boost) + 1.2 #sp lockout
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmgarts
    /// if atks_per_skillactivation > 1 and self.skill == 1:
    /// if self.skill_params[2] > 1:
    /// avghit = (skilldmgarts + (atks_per_skillactivation - 1) * hitdmgarts) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmgarts + int(atks_per_skillactivation) * hitdmgarts) / (int(atks_per_skillactivation)+1)
    /// dps = (avghit+defbonusdmg)/(self.atk_interval/(1+aspd/100))
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval * self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-newres/100), final_atk * 0.05)
    /// defbonusdmg = np.fmax(defense * bonus_arts_scaling *(1-newres/100), defense * bonus_arts_scaling * 0.05)
    /// dps = (hitdmgarts + defbonusdmg)/atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// defbonusdmg = np.fmax(defense * bonus_arts_scaling *(1-newres/100), defense * bonus_arts_scaling * 0.05)
    /// dps = (hitdmg + defbonusdmg)/self.atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut skill_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atks_per_skillactivation: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut defbonusdmg: f64 = 0.0;

        newres = if ((self.unit.module_index as f64) as f64) == 1.0 { ((0) as f64).max((res-10.0) as f64) } else { res };
        let mut bonus_arts_scaling = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        if (self.unit.module_index as f64) == 1.0 && (self.unit.module_level as f64) > 1.0 && self.unit.talent_damage { bonus_arts_scaling = self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0); }
        atkbuff = if ((self.unit.elite as f64) as f64) == 2.0 && self.unit.talent2_damage { self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        aspd = if ((self.unit.elite as f64) as f64) == 2.0 && self.unit.talent2_damage { self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0) } else { 0.0 };
        if (self.unit.skill_index as f64) < 2.0 {
        sp_cost = (self.unit.skill_cost as f64);
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk * (1.0 -newres/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut skilldmgarts = ((final_atk * skill_scale *(1.0 -newres/ 100.0)) as f64).max((final_atk * skill_scale * 0.05) as f64);
        if (self.unit.skill_index as f64) == 0.0 { skilldmgarts = hitdmgarts; }
        defbonusdmg = ((defense * bonus_arts_scaling *(1.0 -newres/ 100.0)) as f64).max((defense * bonus_arts_scaling * 0.05) as f64);
        let mut atkcycle = (self.unit.attack_interval as f64)/(self.unit.attack_speed+aspd)* 100.0;
        if (self.unit.module_index as f64) == 2.0 && self.unit.module_damage {
        sp_cost = sp_cost / (1.0 + 1.0/atkcycle + (self.unit.sp_boost as f64)) + 1.2;
        } else {
        sp_cost = sp_cost /(1.0 + (self.unit.sp_boost as f64)) + 1.2;
        }
        atks_per_skillactivation = sp_cost / atkcycle;
        avghit = skilldmgarts;
        if atks_per_skillactivation > 1.0 && (self.unit.skill_index as f64) == 1.0 {
        if self.unit.skill_parameters[2] > 1.0 {
        avghit = (skilldmgarts + (atks_per_skillactivation - 1.0) * hitdmgarts) / atks_per_skillactivation;
        } else {
        avghit = (skilldmgarts + ((atks_per_skillactivation) as f64).trunc() * hitdmgarts) / (((atks_per_skillactivation) as f64).trunc()+1.0);
        }
        }
        dps = (avghit+defbonusdmg)/((self.unit.attack_interval as f64)/(1.0 +aspd/ 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
        atk_interval = (self.unit.attack_interval as f64) * self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk *(1.0 -newres/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        defbonusdmg = ((defense * bonus_arts_scaling *(1.0 -newres/ 100.0)) as f64).max((defense * bonus_arts_scaling * 0.05) as f64);
        dps = (hitdmgarts + defbonusdmg)/atk_interval * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
        atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        defbonusdmg = ((defense * bonus_arts_scaling *(1.0 -newres/ 100.0)) as f64).max((defense * bonus_arts_scaling * 0.05) as f64);
        dps = (hitdmg + defbonusdmg)/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Ceobe {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ceobe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ceobe {
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
