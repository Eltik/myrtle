//! DPS calculations for Minimalist
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Minimalist operator implementation
pub struct Minimalist {
    pub unit: OperatorUnit,
}

impl Minimalist {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("trait", "minDroneDmg", true, &[], &[], 0, 0)];

    /// Creates a new Minimalist operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// drone_dmg = 1.2 if self.module == 2 else 1.1
    /// if not self.trait_dmg: drone_dmg = 0.2
    /// crate = self.talent1_params[0] if self.elite > 0 else 0
    /// cdmg = self.talent1_params[1]
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill) + self.buff_atk_flat
    /// dmgperinterval = final_atk + drone_dmg * final_atk
    /// hitdmgarts = np.fmax(dmgperinterval * (1-res/100), dmgperinterval * 0.05) * (1 + crate*(cdmg-1))
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed + self.skill_params[1] * self.skill) / 100
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// skill_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// dmgperinterval = final_atk + drone_dmg * final_atk
    /// hitdmg = np.fmax(dmgperinterval * (1-res/100), dmgperinterval * 0.05) * (1 + crate*(cdmg-1))
    /// skilldmg = hitdmg * skill_scale * 2
    /// if not self.trait_dmg: skilldmg *= 2.55/2.4 #because it hits twice, the second hit is guaranteed to hit for more
    /// atkcycle = self.atk_interval/((self.attack_speed)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[1] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval*(self.attack_speed)/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut avghit: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut dmgperinterval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;

        let mut drone_dmg = if ((self.unit.module_index as f64) as f64) == 2.0 { 1.2 } else { 1.1 };
        if !self.unit.trait_damage { drone_dmg = 0.2; }
        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        cdmg = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        if (self.unit.skill_index as f64) < 2.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) * (self.unit.skill_index as f64)) + self.unit.buff_atk_flat;
        dmgperinterval = final_atk + drone_dmg * final_atk;
        hitdmgarts = ((dmgperinterval * (1.0 -res/ 100.0)) as f64).max((dmgperinterval * 0.05) as f64) * (1.0 + crit_rate*(cdmg-1.0));
        dps = hitdmgarts/(self.unit.attack_interval as f64) * (self.unit.attack_speed + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0) * (self.unit.skill_index as f64)) / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        sp_cost = (self.unit.skill_cost as f64)/(1.0 +(self.unit.sp_boost as f64)) + 1.2;
        dmgperinterval = final_atk + drone_dmg * final_atk;
        hitdmg = ((dmgperinterval * (1.0 -res/ 100.0)) as f64).max((dmgperinterval * 0.05) as f64) * (1.0 + crit_rate*(cdmg-1.0));
        skilldmg = hitdmg * skill_scale * 2.0;
        if !self.unit.trait_damage { skilldmg *= 2.55/2.4; }
        let mut atkcycle = (self.unit.attack_interval as f64)/((self.unit.attack_speed)/ 100.0);
        let mut atks_per_skillactivation = sp_cost / atkcycle;
        avghit = skilldmg;
        if atks_per_skillactivation > 1.0 {
        if self.unit.skill_parameters[1] > 1.0 {
        avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg) / atks_per_skillactivation;
        } else {
        avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg) / (((atks_per_skillactivation) as f64).trunc()+1.0);
        }
        }
        dps = avghit/(self.unit.attack_interval as f64)*(self.unit.attack_speed)/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Minimalist {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Minimalist {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Minimalist {
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
