//! DPS calculations for Degenbrecher
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Degenbrecher operator implementation
pub struct Degenbrecher {
    pub unit: OperatorUnit,
}

impl Degenbrecher {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Degenbrecher operator
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
    /// newdef = defense * (1 - self.talent2_params[0]) if self.elite == 2 else defense
    /// dmg = 1.1 if self.module == 1 else 1
    /// atk_scale = self.talent1_params[1] if self.elite > 0 else 1
    ///
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * 2
    /// hitdmg_crit = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05) * 2
    /// hitdmg_tremble = np.fmax(final_atk - newdef, final_atk * 0.05) * 2
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * dmg * 2
    /// skilldmg_crit = np.fmax(final_atk * skill_scale * atk_scale - newdef, final_atk * skill_scale * atk_scale * 0.05) * dmg * 2
    /// skilldmg_tremble = np.fmax(final_atk * skill_scale - newdef, final_atk * skill_scale * 0.05) * dmg * 2
    /// crate = 0 if self.elite == 0 else self.talent1_params[0]
    /// relevant_attack_count = int(5/(self.atk_interval / self.attack_speed * 100)) * 2 #tremble lasts 5 seconds
    /// chance_that_no_crit_occured = (1-crate) ** relevant_attack_count
    /// avghit = hitdmg_crit * crate + hitdmg * (1-crate) * chance_that_no_crit_occured + hitdmg_tremble * (1-crate) * (1 - chance_that_no_crit_occured)
    /// avgskill = skilldmg_crit * crate + skilldmg * (1-crate) * chance_that_no_crit_occured + skilldmg_tremble * (1-crate) * (1 - chance_that_no_crit_occured) * min(self.targets,self.skill_params[1])
    /// if self.skill == 0: avgskill = avghit
    /// average = (self.skill_cost * avghit + avgskill)/(self.skill_cost + 1)
    /// dps = average/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[2]
    /// last_scale = self.skill_params[6]
    /// hitdmg1 = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * atk_scale * skill_scale * 0.05) * dmg
    /// hitdmg2 = np.fmax(final_atk * atk_scale * last_scale - newdef, final_atk * atk_scale * last_scale * 0.05) * dmg
    /// dps = (10 * hitdmg1 + hitdmg2) * min(self.targets,self.skill_params[1])
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens,
        clippy::if_same_then_else,
        clippy::nonminimal_bool,
        clippy::overly_complex_bool_expr,
        clippy::needless_return,
        clippy::collapsible_else_if,
        clippy::neg_multiply,
        clippy::assign_op_pattern,
        clippy::eq_op
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut avghit: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        let mut newdef = if ((self.unit.elite as f64) as f64) == 2.0 {
            defense * (1.0 - self.unit.talent2_parameters.first().copied().unwrap_or(0.0))
        } else {
            defense
        };
        let mut dmg = if ((self.unit.module_index as f64) as f64) == 1.0 {
            1.1
        } else {
            1.0
        };
        atk_scale = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * 2.0;
            let mut hitdmg_crit = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * 2.0;
            let mut hitdmg_tremble =
                ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64) * 2.0;
            skilldmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg
                * 2.0;
            let mut skilldmg_crit = ((final_atk * skill_scale * atk_scale - newdef) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64)
                * dmg
                * 2.0;
            let mut skilldmg_tremble = ((final_atk * skill_scale - newdef) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg
                * 2.0;
            let mut crit_rate = if ((self.unit.elite as f64) as f64) == 0.0 {
                0.0
            } else {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            };
            let mut relevant_attack_count = ((5.0
                / ((self.unit.attack_interval as f64) / self.unit.attack_speed * 100.0) as f64)
                .trunc())
                * 2.0;
            let mut chance_that_no_crit_occured =
                ((1.0 - crit_rate) as f64).powf(relevant_attack_count as f64);
            avghit = hitdmg_crit * crit_rate
                + hitdmg * (1.0 - crit_rate) * chance_that_no_crit_occured
                + hitdmg_tremble * (1.0 - crit_rate) * (1.0 - chance_that_no_crit_occured);
            let mut avgskill = skilldmg_crit * crit_rate
                + skilldmg * (1.0 - crit_rate) * chance_that_no_crit_occured
                + skilldmg_tremble
                    * (1.0 - crit_rate)
                    * (1.0 - chance_that_no_crit_occured)
                    * ((self.unit.targets as f64) as f64)
                        .min((self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                avgskill = avghit;
            }
            let mut average = ((self.unit.skill_cost as f64) * avghit + avgskill)
                / ((self.unit.skill_cost as f64) + 1.0);
            dps = average / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            let mut last_scale = self.unit.skill_parameters.get(6).copied().unwrap_or(0.0);
            let mut hitdmg1 = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64)
                * dmg;
            let mut hitdmg2 = ((final_atk * atk_scale * last_scale - newdef) as f64)
                .max((final_atk * atk_scale * last_scale * 0.05) as f64)
                * dmg;
            dps = (10.0 * hitdmg1 + hitdmg2)
                * ((self.unit.targets as f64) as f64)
                    .min((self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Degenbrecher {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Degenbrecher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
