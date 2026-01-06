//! DPS calculations for AmiyaGuard
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// AmiyaGuard operator implementation
pub struct AmiyaGuard {
    pub unit: OperatorUnit,
}

impl AmiyaGuard {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("module", "NotBlocking", false, &[], &[1], 0, 0), ("skill", "3kills", false, &[], &[], 0, 0)];

    /// Creates a new AmiyaGuard operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// atkbuff = self.talent1_params[0] * (1 + min(1, self.skill))
    /// aspd = 8 if self.module == 1 and self.module_dmg else 0
    /// if self.skill < 2:
    /// atkbuff += self.skill_params[0] * self.skill
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-res/100), final_atk * 0.05)
    /// dps = (1 + self.skill) * hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// if self.skill_dmg:
    /// atkbuff += 3 * self.skill_params[3]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// dps = final_atk/(self.atk_interval/((self.attack_speed+aspd)/100)) * np.fmax(1,-defense) #this defense part has to be included
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atkbuff: f64 = 0.0;

        atkbuff = self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0) * (1.0 + ((1) as f64).min(((self.unit.skill_index as f64)) as f64));
        aspd = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage { 8.0 } else { 0.0 };
        if (self.unit.skill_index as f64) < 2.0 {
        atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) * (self.unit.skill_index as f64);
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmgarts = ((final_atk *(1.0 -res/ 100.0)) as f64).max((final_atk * 0.05) as f64);
        dps = (1.0 + (self.unit.skill_index as f64)) * hitdmgarts/(self.unit.attack_interval as f64) * (self.unit.attack_speed+aspd)/ 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        if self.unit.skill_damage {
        atkbuff += 3.0 * self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
        }
        final_atk = self.unit.atk * (1.0 +atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        dps = final_atk/((self.unit.attack_interval as f64)/((self.unit.attack_speed+aspd)/ 100.0)) * ((1) as f64).max((-defense) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for AmiyaGuard {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for AmiyaGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for AmiyaGuard {
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
