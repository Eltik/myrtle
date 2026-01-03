//! DPS calculations for Pozemka
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Pozemka operator implementation
pub struct Pozemka {
    pub unit: OperatorUnit,
}

impl Pozemka {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Pozemka operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// defshred = 0
    /// if self.talent_dmg:
    /// if self.talent2_dmg:
    /// defshred = 0.25 if self.pot > 4 else 0.23
    /// else:
    /// defshred = 0.2 if self.pot > 4 else 0.18
    /// if self.module == 1:
    /// defshred += 0.05 * (self.module_lvl - 1)
    /// newdef = defense * (1-defshred)
    /// atk_scale = 1.05 if self.module_dmg and self.module == 2 else 1
    /// 
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]*self.skill) + self.buff_atk_flat
    /// rate = self.skill_params[1] if self.skill == 1 else 0
    /// skill_scale = self.skill_params[2] if self.skill == 1 else 0
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// hitdmg2 = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * atk_scale * skill_scale * 0.05)
    /// avghit = rate * hitdmg2 + (1 - rate) * hitdmg
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    /// if self.talent_dmg and self.elite > 0:
    /// final_atk2 = self.drone_atk * (1 + self.skill_params[0]*self.skill)
    /// hitdmg = np.fmax(final_atk2 * atk_scale - newdef, final_atk2 * atk_scale * 0.05)
    /// hitdmg2 = np.fmax(final_atk2 * atk_scale * skill_scale - newdef, final_atk2 * atk_scale * skill_scale * 0.05)
    /// avghit = rate * hitdmg2 + (1 - rate) * hitdmg
    /// dps += avghit/self.drone_atk_interval
    /// 
    /// if self.skill == 3:
    /// self.atk_interval = 1
    /// skill_scale = self.skill_params[1]
    /// skill_scale2 = self.skill_params[2]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * atk_scale * skill_scale * 0.05)
    /// if self.module_dmg or self.skill_dmg:
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale2 - newdef, final_atk * atk_scale * skill_scale2 * 0.05)
    /// hitdmgTW = 0
    /// if self.talent_dmg:
    /// hitdmgTW = np.fmax(self.drone_atk * skill_scale2 - newdef, self.drone_atk * skill_scale2 * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + hitdmgTW
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut hitdmg_tw: f64 = 0.0;
        let mut hitdmg2: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut final_atk2: f64 = 0.0;

        defshred = 0.0;
        if self.unit.talent_damage {
        if self.unit.talent2_damage {
        defshred = if (self.unit.potential as f64) > 4.0 { 0.25 } else { 0.23 };
        } else {
        defshred = if (self.unit.potential as f64) > 4.0 { 0.2 } else { 0.18 };
        }
        if (self.unit.module_index as f64) == 1.0 {
        defshred += 0.05 * ((self.unit.module_level as f64) - 1.0);
        }
        }
        let mut newdef = defense * (1.0 -defshred);
        atk_scale = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 2.0 { 1.05 } else { 1.0 };
        if (self.unit.skill_index as f64) < 2.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[0]*(self.unit.skill_index as f64)) + self.unit.buff_atk_flat;
        let mut rate = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[1] } else { 0.0 };
        skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[2] } else { 0.0 };
        hitdmg = ((final_atk * atk_scale - newdef) as f64).max((final_atk * atk_scale * 0.05) as f64);
        hitdmg2 = ((final_atk * atk_scale * skill_scale - newdef) as f64).max((final_atk * atk_scale * skill_scale * 0.05) as f64);
        avghit = rate * hitdmg2 + (1.0 - rate) * hitdmg;
        dps = avghit/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
        final_atk2 = self.unit.drone_atk * (1.0 + self.unit.skill_parameters[0]*(self.unit.skill_index as f64));
        hitdmg = ((final_atk2 * atk_scale - newdef) as f64).max((final_atk2 * atk_scale * 0.05) as f64);
        hitdmg2 = ((final_atk2 * atk_scale * skill_scale - newdef) as f64).max((final_atk2 * atk_scale * skill_scale * 0.05) as f64);
        avghit = rate * hitdmg2 + (1.0 - rate) * hitdmg;
        dps += avghit/(self.unit.drone_atk_interval as f64);
        }
        }
        if (self.unit.skill_index as f64) == 3.0 {
        // UNTRANSLATED: self.atk_interval = 1
        skill_scale = self.unit.skill_parameters[1];
        let mut skill_scale2 = self.unit.skill_parameters[2];
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale * skill_scale - newdef) as f64).max((final_atk * atk_scale * skill_scale * 0.05) as f64);
        if self.unit.module_damage || self.unit.skill_damage {
        hitdmg = ((final_atk * atk_scale * skill_scale2 - newdef) as f64).max((final_atk * atk_scale * skill_scale2 * 0.05) as f64);
        }
        hitdmg_tw = 0.0;
        if self.unit.talent_damage {
        hitdmg_tw = ((self.unit.drone_atk * skill_scale2 - newdef) as f64).max((self.unit.drone_atk * skill_scale2 * 0.05) as f64);
        }
        dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0 + hitdmg_tw;
        }
        return dps;
    }
}

impl std::ops::Deref for Pozemka {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Pozemka {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
