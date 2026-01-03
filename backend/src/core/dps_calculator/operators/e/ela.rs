//! DPS calculations for Ela
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Ela operator implementation
pub struct Ela {
    pub unit: OperatorUnit,
}

impl Ela {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[3];

    /// Creates a new Ela operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            3, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// if self.elite > 1:
    /// if self.talent2_params[0] > 1:
    /// cdmg = self.talent2_params[0]
    /// crate = self.talent2_params[1]
    /// else:
    /// cdmg = self.talent2_params[1]
    /// crate = self.talent2_params[0]
    /// if self.talent2_dmg:
    /// crate = 1.0
    /// else:
    /// crate = 0
    /// cdmg = 1
    /// 
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg/self.atk_interval * self.attack_speed/100
    /// 
    /// if self.skill == 2:
    /// defshred = self.skill_params[3]
    /// newdef = np.fmax(0, defense - defshred)
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - newdef, final_atk * cdmg * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg/self.atk_interval * self.attack_speed/100
    /// 
    /// if self.skill == 3:
    /// fragile = self.skill_params[3]
    /// if not self.talent2_dmg: fragile = 0
    /// fragile = max(fragile, self.buff_fragile)
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[5]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * (1+fragile)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05) * (1+fragile)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg/0.5 * self.attack_speed/100 /(1+self.buff_fragile)
    /// 
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut crit_rate: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut critdmg: f64 = 0.0;

        if (self.unit.elite as f64) > 1.0 {
        if self.unit.talent2_parameters[0] > 1.0 {
        cdmg = self.unit.talent2_parameters[0];
        crit_rate = self.unit.talent2_parameters[1];
        } else {
        cdmg = self.unit.talent2_parameters[1];
        crit_rate = self.unit.talent2_parameters[0];
        }
        if self.unit.talent2_damage {
        crit_rate = 1.0;
        }
        } else {
        crit_rate = 0.0;
        cdmg = 1.0;
        }
        if (self.unit.skill_index as f64) < 2.0 {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        critdmg = ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
        avgdmg = crit_rate * critdmg + (1.0 -crit_rate) * hitdmg;
        dps = avgdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        defshred = self.unit.skill_parameters[3];
        let mut newdef = ((0) as f64).max((defense - defshred) as f64);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
        critdmg = ((final_atk * cdmg - newdef) as f64).max((final_atk * cdmg * 0.05) as f64);
        avgdmg = crit_rate * critdmg + (1.0 -crit_rate) * hitdmg;
        dps = avgdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
        let mut fragile = self.unit.skill_parameters[3];
        if !self.unit.talent2_damage { fragile = 0.0; }
        fragile = ((fragile) as f64).max((self.unit.buff_fragile) as f64);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters[5]) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * (1.0 +fragile);
        critdmg = ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64) * (1.0 +fragile);
        avgdmg = crit_rate * critdmg + (1.0 -crit_rate) * hitdmg;
        dps = avgdmg/0.5 * self.unit.attack_speed/ 100.0 /(1.0 +self.unit.buff_fragile);
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    /// 
    /// if self.skill == 3:
    /// return(self.skill_dps(defense,res) * 40 * (0.5/(self.attack_speed/100)))
    /// else:
    /// return(super().total_dmg(defense,res))
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        if (self.unit.skill_index as f64) == 3.0 {
        // UNTRANSLATED: return(self.skill_dps(defense,res) * 40 * (0.5/(self.attack_speed/100))) - method calls need manual implementation
        0.0 // placeholder
        } else {
        // UNTRANSLATED: return(super().total_dmg(defense,res)) - method calls need manual implementation
        0.0 // placeholder
        }
    }
}

impl std::ops::Deref for Ela {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ela {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
