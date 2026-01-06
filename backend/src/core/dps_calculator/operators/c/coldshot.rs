//! DPS calculations for Coldshot
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Coldshot operator implementation
pub struct Coldshot {
    pub unit: OperatorUnit,
}

impl Coldshot {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("trait", "outOfAmmo", true, &[], &[], 0, 0)];

    /// Creates a new Coldshot operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );



        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// 
    /// ammo = 4 + 2 * self.elite
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// atk_scale = 1.2
    /// talent_scale = self.talent1_params[0] if self.elite > 0 else 1
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// reload_time = 2.4 if self.skill == 2 else 1.6
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// hitdmg2 = np.fmax(final_atk * atk_scale * talent_scale - defense, final_atk * atk_scale * talent_scale * 0.05)
    /// if self.atk_interval/self.attack_speed*100 >= 2: hitdmg = hitdmg2 #if attacks are so slow that the talent actually activates
    /// 
    /// if self.trait_dmg: #full clip
    /// if self.talent_dmg or self.atk_interval/self.attack_speed*100 >= 2:
    /// dps = (hitdmg * (ammo -1) + hitdmg2) / ammo / self.atk_interval * self.attack_speed/100
    /// else:
    /// dps = hitdmg2 / 2
    /// else:
    /// if self.module != 1:
    /// dps = hitdmg2/(self.atk_interval/self.attack_speed*100 + reload_time)
    /// else:
    /// if self.atk_interval/self.attack_speed*100 >= 2:
    /// dps = hitdmg2 * 2 /(self.atk_interval/self.attack_speed*100 * 2 + reload_time)
    /// else:
    /// dps = (hitdmg2 + hitdmg) /(self.atk_interval/self.attack_speed*100 * 2 + reload_time)
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut ammo = 4.0 + 2.0 * (self.unit.elite as f64);
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 { self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        let mut atk_scale = 1.2;
        let mut talent_scale = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0) } else { 1.0 };
        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut reload_time = if ((self.unit.skill_index as f64) as f64) == 2.0 { 2.4 } else { 1.6 };
        let mut hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut hitdmg2 = ((final_atk * atk_scale * talent_scale - defense) as f64).max((final_atk * atk_scale * talent_scale * 0.05) as f64);
        if (self.unit.attack_interval as f64)/self.unit.attack_speed*100.0 >= 2.0 { hitdmg = hitdmg2; }
        if self.unit.trait_damage { // full clip
        if self.unit.talent_damage || (self.unit.attack_interval as f64)/self.unit.attack_speed*100.0 >= 2.0 {
        dps = (hitdmg * (ammo -1.0) + hitdmg2) / ammo / (self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        } else {
        dps = hitdmg2 / 2.0;
        }
        } else {
        if (self.unit.module_index as f64) != 1.0 {
        dps = hitdmg2/((self.unit.attack_interval as f64)/self.unit.attack_speed*100.0 + reload_time);
        } else {
        if (self.unit.attack_interval as f64)/self.unit.attack_speed*100.0 >= 2.0 {
        dps = hitdmg2 * 2.0 /((self.unit.attack_interval as f64)/self.unit.attack_speed*100.0 * 2.0 + reload_time);
        } else {
        dps = (hitdmg2 + hitdmg) /((self.unit.attack_interval as f64)/self.unit.attack_speed*100.0 * 2.0 + reload_time);
        }
        }
        }
        return dps;
    }
}

impl std::ops::Deref for Coldshot {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Coldshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Coldshot {
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
