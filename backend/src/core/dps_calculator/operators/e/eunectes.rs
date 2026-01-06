//! DPS calculations for Eunectes
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Eunectes operator implementation
pub struct Eunectes {
    pub unit: OperatorUnit,
}

impl Eunectes {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("talent", "<50%hp", true, &[], &[], 1, 0), ("module", "WhileBlocking", false, &[], &[2], 0, 0)];

    /// Creates a new Eunectes operator
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
    /// atk_scale = self.talent1_params[2] if self.talent_dmg and self.elite > 0 else 1
    /// atkbuff = 0.15 if self.module_dmg and self.module == 2 else 0
    /// final_atk = self.atk *(1+ self.buff_atk + atkbuff + self.skill_params[0] * min(self.skill,1)) + self.buff_atk_flat
    /// atk_interval = 2 if self.skill == 2 else self.atk_interval
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/atk_interval * self.attack_speed/100
    /// block = 3 if self.skill == 3 else 1
    /// if self.module == 3 and self.skill > 0:
    /// dps *= min(self.targets, block + (self.module_lvl - 1))
    /// dps *= 0.8 + 0.2 * self.module_lvl
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut atk_scale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0) } else { 1.0 };
        let mut atkbuff = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 2.0 { 0.15 } else { 0.0 };
        let mut final_atk = self.unit.atk *(1.0 + self.unit.buff_atk + atkbuff + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) * (((self.unit.skill_index as f64)) as f64).min((1) as f64)) + self.unit.buff_atk_flat;
        atk_interval = if ((self.unit.skill_index as f64) as f64) == 2.0 { 2.0 } else { (self.unit.attack_interval as f64) };
        let mut hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut dps = hitdmg/atk_interval * self.unit.attack_speed/ 100.0;
        let mut block = if ((self.unit.skill_index as f64) as f64) == 3.0 { 3.0 } else { 1.0 };
        if (self.unit.module_index as f64) == 3.0 && (self.unit.skill_index as f64) > 0.0 {
        dps *= (((self.unit.targets as f64)) as f64).min((block + ((self.unit.module_level as f64) - 1.0)) as f64);
        dps *= 0.8 + 0.2 * (self.unit.module_level as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Eunectes {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Eunectes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Eunectes {
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
