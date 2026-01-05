//! DPS calculations for Meteor
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Meteor operator implementation
pub struct Meteor {
    pub unit: OperatorUnit,
}

impl Meteor {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Meteor operator
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
    /// atk_scale = 1.1 if self.module == 1 and self.talent_dmg else 1
    /// talentscale = self.talent1_params[0] if self.talent_dmg and self.elite > 0 else 1
    ///
    /// if self.skill < 2:
    /// sp_cost = self.skill_cost
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// defshred = self.skill_params[1] if self.skill == 1 else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmglow = np.fmax(final_atk * atk_scale * talentscale - defense, final_atk * atk_scale * talentscale * 0.05)
    /// hitdmg = np.fmax(final_atk * atk_scale * talentscale - defense * (1+defshred), final_atk * atk_scale * talentscale * 0.05)
    /// reapply_duration = (self.skill_cost+1) * self.atk_interval / self.attack_speed * 100
    /// avghitdmg = hitdmg * min(1, 5/reapply_duration) + hitdmglow * (1- min(1, 5/reapply_duration))
    /// skilldmg = np.fmax(final_atk * atk_scale * talentscale * skill_scale - defense * (1+defshred), final_atk * atk_scale * talentscale * skill_scale * 0.05)
    /// avgdmg = (sp_cost * avghitdmg + skilldmg) / (sp_cost + 1)
    /// dps = avgdmg/self.atk_interval * self.attack_speed/100
    ///
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
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut avgdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.talent_damage {
            1.1
        } else {
            1.0
        };
        let mut talentscale = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0
        {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            sp_cost = (self.unit.skill_cost as f64);
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            defshred = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut hitdmglow = ((final_atk * atk_scale * talentscale - defense) as f64)
                .max((final_atk * atk_scale * talentscale * 0.05) as f64);
            hitdmg = ((final_atk * atk_scale * talentscale - defense * (1.0 + defshred)) as f64)
                .max((final_atk * atk_scale * talentscale * 0.05) as f64);
            let mut reapply_duration = ((self.unit.skill_cost as f64) + 1.0)
                * (self.unit.attack_interval as f64)
                / self.unit.attack_speed
                * 100.0;
            let mut avghitdmg = hitdmg * ((1) as f64).min((5.0 / reapply_duration) as f64)
                + hitdmglow * (1.0 - ((1) as f64).min((5.0 / reapply_duration) as f64));
            skilldmg = ((final_atk * atk_scale * talentscale * skill_scale
                - defense * (1.0 + defshred)) as f64)
                .max((final_atk * atk_scale * talentscale * skill_scale * 0.05) as f64);
            avgdmg = (sp_cost * avghitdmg + skilldmg) / (sp_cost + 1.0);
            dps = avgdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Meteor {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Meteor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
