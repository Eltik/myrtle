//! DPS calculations for Conviction
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Conviction operator implementation
pub struct Conviction {
    pub unit: OperatorUnit,
}

impl Conviction {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Conviction operator
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
    /// atk_scale = 1.15 if self.module == 1 and self.module_dmg else 1
    ///
    /// if self.skill < 2:
    /// skill_scale = (self.skill_params[0]-1) * self.skill + 1
    /// skill_scale2 = (self.skill_params[3]-1) * self.skill + 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg1 = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk* atk_scale * skill_scale * 0.05)
    /// skilldmg2 = np.fmax(final_atk * atk_scale * skill_scale2 - defense, final_atk* atk_scale * skill_scale2 * 0.05)
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg1 * 0.95 + skilldmg2 * 0.05
    /// if atks_per_skillactivation > 1:
    /// avghit = (0.95 * skilldmg1 + 0.05 * skilldmg2 + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale * atk_scale * (1-res/100), final_atk * skill_scale * atk_scale * 0.05) * self.targets
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill_dmg:
    /// dps += skilldmg / sp_cost
    /// else:
    /// dps *= (sp_cost -5)/sp_cost
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

        let mut atk_interval: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale =
                (self.unit.skill_parameters[0] - 1.0) * (self.unit.skill_index as f64) + 1.0;
            let mut skill_scale2 =
                (self.unit.skill_parameters[3] - 1.0) * (self.unit.skill_index as f64) + 1.0;
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut skilldmg1 = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            let mut skilldmg2 = ((final_atk * atk_scale * skill_scale2 - defense) as f64)
                .max((final_atk * atk_scale * skill_scale2 * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg1 * 0.95 + skilldmg2 * 0.05;
            if atks_per_skillactivation > 1.0 {
                avghit = (0.95 * skilldmg1
                    + 0.05 * skilldmg2
                    + (atks_per_skillactivation - 1.0) * hitdmg)
                    / atks_per_skillactivation;
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[1];
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64)
                * (self.unit.targets as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if self.unit.skill_damage {
                dps += skilldmg / sp_cost;
            } else {
                dps *= (sp_cost - 5.0) / sp_cost;
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Conviction {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Conviction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
