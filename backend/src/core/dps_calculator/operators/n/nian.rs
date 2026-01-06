//! DPS calculations for Nian
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Nian operator implementation
pub struct Nian {
    pub unit: OperatorUnit,
}

impl Nian {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("module", "3shieldsBroken", false, &[], &[1], 0, 2)];

    /// Creates a new Nian operator
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
    /// atkbuff = 0
    /// if self.module == 1 and self.module_dmg and self.module_lvl > 1:
    /// atkbuff += 3 * 0.05 if self.module_lvl == 2 else 3 * 0.07
    /// 
    /// if self.skill == 1:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[2]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmg * self.hits
    /// if self.skill in [0,3]:
    /// atkbuff += self.skill_params[4] if self.skill == 3 else 0
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atkbuff: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atkbuff = 0.0;
        if (self.unit.module_index as f64) == 1.0 && self.unit.module_damage && (self.unit.module_level as f64) > 1.0 {
        atkbuff += if ((self.unit.module_level as f64) as f64) == 2.0 { 3.0 * 0.05 } else { 3.0 * 0.07 };
        }
        if (self.unit.skill_index as f64) == 1.0 {
        atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * (1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        atk_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale * (1.0 -res/ 100.0)) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg * 0.0 /* self.hits - defaults to 0 */;
        }
        if [0.0, 3.0].contains(&(((self.unit.skill_index as f64)) as f64)) {
        atkbuff += if ((self.unit.skill_index as f64) as f64) == 3.0 { self.unit.skill_parameters.get(4).copied().unwrap_or(0.0) } else { 0.0 };
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Nian {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Nian {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Nian {
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
