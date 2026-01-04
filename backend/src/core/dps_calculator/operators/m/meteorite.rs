//! DPS calculations for Meteorite
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Meteorite operator implementation
pub struct Meteorite {
    pub unit: OperatorUnit,
}

impl Meteorite {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Meteorite operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// crate = self.talent1_params[1] if self.elite > 0 else 0
    /// newdef = np.fmax(0, defense - 100) if self.module == 2 else defense
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// final_atk_crit = self.atk * (1 + self.buff_atk + 0.6) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// hitcrit = np.fmax(final_atk_crit - newdef, final_atk_crit * 0.05)
    /// skill_scale = self.skill_params[0] if self.skill > 0 else 1
    /// if self.skill < 2:
    /// skillhitdmg = np.fmax(final_atk * skill_scale - newdef, final_atk * skill_scale * 0.05)
    /// skillcritdmg = np.fmax(final_atk_crit *skill_scale - newdef, final_atk_crit * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// avghit = crate * hitcrit + (1-crate) * hitdmg
    /// avgskill = crate * skillcritdmg + (1-crate) * skillhitdmg
    /// avgphys = (sp_cost * avghit + avgskill) / (sp_cost + 1)
    /// dps = avgphys/self.atk_interval * self.attack_speed/100 * self.targets
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

        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut newdef = if ((self.unit.module_index as f64) as f64) == 2.0 {
            ((0) as f64).max((defense - 100.0) as f64)
        } else {
            defense
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut final_atk_crit =
            self.unit.atk * (1.0 + self.unit.buff_atk + 0.6) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
        let mut hitcrit = ((final_atk_crit - newdef) as f64).max((final_atk_crit * 0.05) as f64);
        skill_scale = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            let mut skillhitdmg = ((final_atk * skill_scale - newdef) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            let mut skillcritdmg = ((final_atk_crit * skill_scale - newdef) as f64)
                .max((final_atk_crit * skill_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            avghit = crit_rate * hitcrit + (1.0 - crit_rate) * hitdmg;
            let mut avgskill = crit_rate * skillcritdmg + (1.0 - crit_rate) * skillhitdmg;
            avgphys = (sp_cost * avghit + avgskill) / (sp_cost + 1.0);
            dps = avgphys / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Meteorite {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Meteorite {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
