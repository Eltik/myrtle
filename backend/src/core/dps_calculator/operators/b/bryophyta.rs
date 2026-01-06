//! DPS calculations for Bryophyta
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Bryophyta operator implementation
pub struct Bryophyta {
    pub unit: OperatorUnit,
}

impl Bryophyta {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("trait", "blocking", true, &[], &[], 0, 0)];

    /// Creates a new Bryophyta operator
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
    /// atk_scale = 1
    /// if self.trait_dmg:
    /// atk_scale = 1.3 if self.module == 1 else 1.2
    /// 
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk* atk_scale * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1)
    /// dps = avgphys/self.atk_interval * self.attack_speed/100
    /// else:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill/2) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut skill_scale: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;

        atk_scale = 1.0;
        if self.unit.trait_damage {
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 { 1.3 } else { 1.2 };
        }
        if (self.unit.skill_index as f64) == 1.0 {
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut skillhitdmg = ((final_atk * atk_scale * skill_scale - defense) as f64).max((final_atk* atk_scale * skill_scale * 0.05) as f64);
        sp_cost = (self.unit.skill_cost as f64);
        avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1.0);
        dps = avgphys/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        } else {
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) * (self.unit.skill_index as f64)/ 2.0) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg/(self.unit.attack_interval as f64) * self.unit.attack_speed/ 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Bryophyta {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Bryophyta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Bryophyta {
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
