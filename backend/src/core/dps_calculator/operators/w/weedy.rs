//! DPS calculations for Weedy
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Weedy operator implementation
pub struct Weedy {
    pub unit: OperatorUnit,
}

impl Weedy {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Weedy operator
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
    /// atkbuff = 0
    /// if self.module == 1 and self.module_dmg:
    /// if self.module_lvl == 2: atkbuff += 0.15
    /// if self.module_lvl == 3: atkbuff += 0.2
    /// 
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// sp_cost = self.skill_cost/(1+ self.sp_boost) + 1.2
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/(self.atk_interval/(self.attack_speed/100)) * self.targets
    /// 
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/(3.84/(self.attack_speed/100)) * min(self.targets, 2)
    /// 
    /// if self.talent_dmg and self.elite > 0:
    /// summonhit = np.fmax(self.drone_atk - defense, self.drone_atk * 0.05)
    /// dps += summonhit / self.drone_atk_interval
    /// 
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        atkbuff = 0.0;
        if (self.unit.module_index as f64) == 1.0 && self.unit.module_damage {
        // UNTRANSLATED: if self.module_lvl == 2: atkbuff += 0.15
        // UNTRANSLATED: if self.module_lvl == 3: atkbuff += 0.2
        }
        if (self.unit.skill_index as f64) < 2.0 {
        skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[0] } else { 1.0 };
        sp_cost = (self.unit.skill_cost as f64)/(1.0 + (self.unit.sp_boost as f64)) + 1.2;
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        skilldmg = ((final_atk * skill_scale - defense) as f64).max((final_atk * skill_scale * 0.05) as f64);
        let mut atkcycle = (self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0);
        let mut atks_per_skillactivation = sp_cost / atkcycle;
        avghit = skilldmg;
        if atks_per_skillactivation > 1.0 {
        avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg) / (((atks_per_skillactivation) as f64).trunc()+1.0);
        }
        dps = avghit/((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0)) * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
        atkbuff += self.unit.skill_parameters[0];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg/(3.84/(self.unit.attack_speed/ 100.0)) * (((self.unit.targets as f64)) as f64).min((2) as f64);
        }
        if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
        let mut summonhit = ((self.unit.drone_atk - defense) as f64).max((self.unit.drone_atk * 0.05) as f64);
        dps += summonhit / (self.unit.drone_atk_interval as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Weedy {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Weedy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
