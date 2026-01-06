//! DPS calculations for Caper
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Caper operator implementation
pub struct Caper {
    pub unit: OperatorUnit,
}

impl Caper {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[("trait", "maxRange", true, &[], &[], 0, 0)];

    /// Creates a new Caper operator
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
    /// atk_scale = 1.1 if self.module == 1 and self.trait_dmg else 1
    /// crate = self.talent1_params[0] if self.elite > 0 else 0
    /// cdmg = self.talent1_params[1] if self.elite > 0 else 1
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg * atk_scale - defense, final_atk * cdmg * atk_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// skillcritdmg = np.fmax(final_atk * cdmg * skill_scale * atk_scale - defense, final_atk * cdmg * skill_scale * atk_scale * 0.05)
    /// hitdmg = critdmg * crate + (1-crate) * hitdmg
    /// skillhitdmg = skillcritdmg * crate + (1-crate) * skillhitdmg
    /// if self.skill == 0: skillhitdmg = hitdmg
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1)
    /// interval = 20/13.6 if not self.trait_dmg else (self.atk_interval/(self.attack_speed/100)) #source: dr silvergun vid
    /// dps = avgphys/interval
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg * atk_scale - defense, final_atk * cdmg * atk_scale * 0.05)
    /// hitdmg = critdmg * crate + (1-crate) * hitdmg
    /// interval = 20/13.6 if not self.trait_dmg else (self.atk_interval/(self.attack_speed/100))
    /// dps = 2 * hitdmg/interval
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op, clippy::get_first)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut hitdmg: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut interval: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut critdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.trait_damage { 1.1 } else { 1.0 };
        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0) } else { 0.0 };
        cdmg = if ((self.unit.elite as f64) as f64) > 0.0 { self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0) } else { 1.0 };
        if (self.unit.skill_index as f64) < 2.0 {
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        critdmg = ((final_atk * cdmg * atk_scale - defense) as f64).max((final_atk * cdmg * atk_scale * 0.05) as f64);
        let mut skillhitdmg = ((final_atk * skill_scale * atk_scale - defense) as f64).max((final_atk * skill_scale * atk_scale * 0.05) as f64);
        let mut skillcritdmg = ((final_atk * cdmg * skill_scale * atk_scale - defense) as f64).max((final_atk * cdmg * skill_scale * atk_scale * 0.05) as f64);
        hitdmg = critdmg * crit_rate + (1.0 -crit_rate) * hitdmg;
        skillhitdmg = skillcritdmg * crit_rate + (1.0 -crit_rate) * skillhitdmg;
        if (self.unit.skill_index as f64) == 0.0 { skillhitdmg = hitdmg; }
        sp_cost = (self.unit.skill_cost as f64);
        avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1.0);
        interval = if !self.unit.trait_damage { 20.0 /13.6 } else { ((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0)) };
        dps = avgphys/interval;
        }
        if (self.unit.skill_index as f64) == 2.0 {
        atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        critdmg = ((final_atk * cdmg * atk_scale - defense) as f64).max((final_atk * cdmg * atk_scale * 0.05) as f64);
        hitdmg = critdmg * crit_rate + (1.0 -crit_rate) * hitdmg;
        interval = if !self.unit.trait_damage { 20.0 /13.6 } else { ((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0)) };
        dps = 2.0 * hitdmg/interval;
        }
        return dps;
    }
}

impl std::ops::Deref for Caper {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Caper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Caper {
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
