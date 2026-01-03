//! DPS calculations for Virtuosa
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Virtuosa operator implementation
pub struct Virtuosa {
    pub unit: OperatorUnit,
}

impl Virtuosa {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Virtuosa operator
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
    /// ele_gauge = 1000 if self.trait_dmg else 2000
    /// necro_scale = self.talent1_params[0]
    /// necro_fragile = max(self.talent2_params) if self.elite == 2 else 1
    /// ele_fragile = self.talent2_params[0] if self.module == 1 and self.module_lvl > 1 else 1
    /// if self.module == 2: ele_fragile = 1.1
    /// falloutdmg = 12000
    /// if self.module == 2 and self.module_lvl > 1: falloutdmg = 15 * (800 + 50 * self.module_lvl)
    /// 
    /// ####the actual skills
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// necro_skill_scale = self.skill_params[1]
    /// sp_cost = self.skill_cost / (1 + self.sp_boost) + 1.2
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// skilldmg =np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// if skill_scale > 2.05:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/(self.atk_interval/(self.attack_speed/100))
    /// necro_dps = final_atk * necro_scale * necro_fragile
    /// necro_skill_dps = final_atk * necro_skill_scale * necro_fragile / sp_cost
    /// time_to_fallout_1 = ele_gauge / (necro_dps + necro_skill_dps) #this is meant as a rough estimate to her saving skill charges against fallout, potentially improving dps
    /// time_to_fallout = ele_gauge / (necro_dps + necro_skill_dps/(time_to_fallout_1)*(time_to_fallout_1 + 15))
    /// if skill_scale < 2.05: time_to_fallout = time_to_fallout_1
    /// dps += falloutdmg * ele_fragile / (15 + time_to_fallout) / (1 + self.buff_fragile)
    /// if self.targets > 1:
    /// dps += falloutdmg * ele_fragile / (15 + ele_gauge/necro_dps) / (1 + self.buff_fragile) * (self.targets -1)
    /// 
    /// if self.skill == 2:
    /// aspd = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// extra_ele = final_atk * self.skill_params[1]
    /// ele_gauge = ele_gauge / necro_fragile
    /// eleApplicationTarget = final_atk * necro_scale + extra_ele / (self.atk_interval/((self.attack_speed+aspd)/100))
    /// eleApplicationBase = final_atk * necro_scale
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// artsdps = hitdmgarts/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// targetEledps = falloutdmg * ele_fragile / (15 + ele_gauge/eleApplicationTarget)
    /// ambientEledps = falloutdmg * ele_fragile / (15 + ele_gauge/eleApplicationBase)
    /// dps = np.fmin(self.targets, 2) * (artsdps + targetEledps/(1 + self.buff_fragile))
    /// if self.targets > 2:
    /// dps += ambientEledps * (self.targets - 2) / (1 + self.buff_fragile)
    /// 
    /// if self.skill in [0,3]:
    /// if self.skill == 3: necro_fragile = self.skill_params[1] * (necro_fragile - 1) + 1
    /// atkbuff = self.skill_params[0]
    /// atkbuff += self.skill_params[3] if self.skill_dmg else 0
    /// if self.skill == 0: atkbuff = 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// necro_dps = final_atk * necro_scale * necro_fragile
    /// time_to_fallout = ele_gauge / necro_dps
    /// dps = self.targets * 12000 * ele_fragile / (15 + time_to_fallout) * np.fmax(1,-defense) / (1 + self.buff_fragile)
    /// if self.skill == 0:
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps += hitdmgarts/(self.atk_interval/(self.attack_speed/100))
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut skilldmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut time_to_fallout_1: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut necro_skill_dps: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut time_to_fallout: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut necro_dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        let mut ele_gauge = if self.unit.trait_damage { 1000.0 } else { 2000.0 };
        let mut necro_scale = self.unit.talent1_parameters[0];
        let mut necro_fragile = if ((self.unit.elite as f64) as f64) == 2.0 { self.unit.talent2_parameters.iter().cloned().fold(f64::NEG_INFINITY, f64::max) } else { 1.0 };
        let mut ele_fragile = if ((self.unit.module_index as f64) as f64) == 1.0 && ((self.unit.module_level as f64) as f64) > 1.0 { self.unit.talent2_parameters[0] } else { 1.0 };
        if (self.unit.module_index as f64) == 2.0 { ele_fragile = 1.1; }
        let mut falloutdmg = 12000.0;
        if (self.unit.module_index as f64) == 2.0 && (self.unit.module_level as f64) > 1.0 { falloutdmg = 15.0 * (800.0 + 50.0 * (self.unit.module_level as f64)); }
        // ###the actual skills
        if (self.unit.skill_index as f64) == 1.0 {
        skill_scale = self.unit.skill_parameters[0];
        let mut necro_skill_scale = self.unit.skill_parameters[1];
        sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * (1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        skilldmg = ((final_atk * skill_scale * (1.0 -res/ 100.0)) as f64).max((final_atk * skill_scale * 0.05) as f64);
        let mut atkcycle = (self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0);
        let mut atks_per_skillactivation = sp_cost / atkcycle;
        avghit = skilldmg;
        if atks_per_skillactivation > 1.0 {
        if skill_scale > 2.05 {
        avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg) / atks_per_skillactivation;
        } else {
        avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg) / (((atks_per_skillactivation) as f64).trunc()+1.0);
        }
        }
        dps = avghit/((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0));
        necro_dps = final_atk * necro_scale * necro_fragile;
        necro_skill_dps = final_atk * necro_skill_scale * necro_fragile / sp_cost;
        time_to_fallout_1 = ele_gauge / (necro_dps + necro_skill_dps);
        time_to_fallout = ele_gauge / (necro_dps + necro_skill_dps/(time_to_fallout_1)*(time_to_fallout_1 + 15.0));
        if skill_scale < 2.05 { time_to_fallout = time_to_fallout_1; }
        dps += falloutdmg * ele_fragile / (15.0 + time_to_fallout) / (1.0 + self.unit.buff_fragile);
        if (self.unit.targets as f64) > 1.0 {
        dps += falloutdmg * ele_fragile / (15.0 + ele_gauge/necro_dps) / (1.0 + self.unit.buff_fragile) * ((self.unit.targets as f64) -1.0);
        }
        }
        if (self.unit.skill_index as f64) == 2.0 {
        aspd = self.unit.skill_parameters[0];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut extra_ele = final_atk * self.unit.skill_parameters[1];
        ele_gauge = ele_gauge / necro_fragile;
        let mut ele_application_target = final_atk * necro_scale + extra_ele / ((self.unit.attack_interval as f64)/((self.unit.attack_speed+aspd)/ 100.0));
        let mut ele_application_base = final_atk * necro_scale;
        hitdmgarts = ((final_atk * (1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut artsdps = hitdmgarts/((self.unit.attack_interval as f64)/((self.unit.attack_speed+aspd)/ 100.0));
        let mut target_eledps = falloutdmg * ele_fragile / (15.0 + ele_gauge/ele_application_target);
        let mut ambient_eledps = falloutdmg * ele_fragile / (15.0 + ele_gauge/ele_application_base);
        dps = (((self.unit.targets as f64)) as f64).min((2) as f64) * (artsdps + target_eledps/(1.0 + self.unit.buff_fragile));
        if (self.unit.targets as f64) > 2.0 {
        dps += ambient_eledps * ((self.unit.targets as f64) - 2.0) / (1.0 + self.unit.buff_fragile);
        }
        }
        if [0.0, 3.0].contains(&(((self.unit.skill_index as f64)) as f64)) {
        if (self.unit.skill_index as f64) == 3.0 { necro_fragile = self.unit.skill_parameters[1] * (necro_fragile - 1.0) + 1.0; }
        atkbuff = self.unit.skill_parameters[0];
        atkbuff += if self.unit.skill_damage { self.unit.skill_parameters[3] } else { 0.0 };
        if (self.unit.skill_index as f64) == 0.0 { atkbuff = 0.0; }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        necro_dps = final_atk * necro_scale * necro_fragile;
        time_to_fallout = ele_gauge / necro_dps;
        dps = (self.unit.targets as f64) * 12000.0 * ele_fragile / (15.0 + time_to_fallout) * ((1) as f64).max((-defense) as f64) / (1.0 + self.unit.buff_fragile);
        if (self.unit.skill_index as f64) == 0.0 {
        hitdmgarts = ((final_atk * (1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        dps += hitdmgarts/((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0));
        }
        }
        return dps;
    }
}

impl std::ops::Deref for Virtuosa {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Virtuosa {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
