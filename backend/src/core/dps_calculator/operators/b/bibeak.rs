//! DPS calculations for Bibeak
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Bibeak operator implementation
pub struct Bibeak {
    pub unit: OperatorUnit,
}

impl Bibeak {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Bibeak operator
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
    /// aspd = self.talent1_params[0] * self.talent1_params[1] if self.talent_dmg else 0
    /// atkbuff = 0.01 * (self.module_lvl-1) * self.talent1_params[1] if self.talent_dmg and self.module == 1 and self.module_lvl > 1 else 0
    /// dmg_multiplier = 1.1 if self.module == 1 else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// skillhitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * dmg_multiplier
    /// skillartsdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_multiplier
    /// if self.skill == 0: skillhitdmg = hitdmg
    /// sp_cost = self.skill_cost
    /// avg_phys = 2 * (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1)
    /// avg_arts = 0 if self.targets == 1 else skillartsdmg / (sp_cost +1) * self.skill
    /// dps = (avg_phys+avg_arts)/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[2]
    /// skillartsdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg_multiplier
    /// avg_hit = (2 * hitdmg * self.skill_cost + skillartsdmg * min(self.targets, self.skill_params[0])) / self.skill_cost
    /// dps = avg_hit/self.atk_interval * (self.attack_speed + aspd)/100
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

        let mut skill_scale: f64 = 0.0;
        let mut skillartsdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        aspd = if self.unit.talent_damage {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
                * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        atkbuff = if self.unit.talent_damage
            && ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            0.01 * (((self.unit.module_level as f64) as f64) - 1.0)
                * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut dmg_multiplier = if ((self.unit.module_index as f64) as f64) == 1.0 {
            1.1
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut skillhitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_multiplier;
            skillartsdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_multiplier;
            if (self.unit.skill_index as f64) == 0.0 {
                skillhitdmg = hitdmg;
            }
            sp_cost = (self.unit.skill_cost as f64);
            let mut avg_phys = 2.0 * (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1.0);
            let mut avg_arts = if (self.unit.targets as f64) == 1.0 {
                0.0
            } else {
                skillartsdmg / (sp_cost + 1.0) * ((self.unit.skill_index as f64) as f64)
            };
            dps = (avg_phys + avg_arts) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            skillartsdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg_multiplier;
            let mut avg_hit = (2.0 * hitdmg * (self.unit.skill_cost as f64)
                + skillartsdmg
                    * ((self.unit.targets as f64) as f64)
                        .min((self.unit.skill_parameters.first().copied().unwrap_or(0.0)) as f64))
                / (self.unit.skill_cost as f64);
            dps = avg_hit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Bibeak {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Bibeak {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
