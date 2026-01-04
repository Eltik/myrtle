//! DPS calculations for Kirara
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Kirara operator implementation
pub struct Kirara {
    pub unit: OperatorUnit,
}

impl Kirara {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Kirara operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skillhitdmg = np.fmax(final_atk * skill_scale * (1 - res/100), final_atk * skill_scale * 0.05)
    /// if self.skill == 0: skillhitdmg = hitdmg
    /// sp_cost = self.skill_cost
    /// avghit = ((sp_cost+1) * hitdmg + skillhitdmg) / (sp_cost + 1) * self.targets
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * skill_scale * (1- res/100), final_atk * skill_scale * 0.05)
    /// dps = hitdmg * self.targets
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
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                skillhitdmg = hitdmg;
            }
            sp_cost = (self.unit.skill_cost as f64);
            avghit = ((sp_cost + 1.0) * hitdmg + skillhitdmg) / (sp_cost + 1.0)
                * (self.unit.targets as f64);
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps = hitdmg * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Kirara {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Kirara {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
