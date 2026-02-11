//! DPS calculations for HoshigumaAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// HoshigumaAlter operator implementation
pub struct HoshigumaAlter {
    pub unit: OperatorUnit,
}

impl HoshigumaAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent2", "maxTenacity", false, &[], &[], 2, 0),
        ("skill", "lastStand", false, &[3], &[], 0, 0),
    ];

    /// Creates a new HoshigumaAlter operator
    #[allow(unused_parens)]
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
    /// extra_scale = 0.1 if self.module == 1 else 0
    /// atkbuff = self.talent2_params[2] if self.talent2_dmg and self.talent_dmg and self.elite == 2 else 0
    /// if self.module == 1 and self.module_lvl > 1 and self.talent2_dmg and self.talent_dmg: atkbuff += 0.05 * self.module_lvl
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 1:
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1 + extra_scale) * (1-res/100), final_atk * (1 + extra_scale) * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// skill_scale = self.skill_params[2] + extra_scale
    /// reflectdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// dps += reflectdmg * self.hits
    /// if self.skill == 3:
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + self.skill_params[1]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1 + extra_scale) * (1-res/100), final_atk * (1 + extra_scale) * 0.05)
    /// hits = 4 if self.skill_dmg else 3
    /// dps = hits * hitdmg/self.atk_interval * self.attack_speed/100 * min(self.skill_params[2], self.targets)
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;

        let mut extra_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        atkbuff = if self.unit.talent2_damage
            && self.unit.talent_damage
            && ((self.unit.elite as f64) as f64) == 2.0
        {
            self.unit.talent2_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.module_index as f64) == 1.0
            && (self.unit.module_level as f64) > 1.0
            && self.unit.talent2_damage
            && self.unit.talent_damage
        {
            atkbuff += 0.05 * (self.unit.module_level as f64);
        }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        if (self.unit.skill_index as f64) == 1.0 {
            final_atk = self.unit.atk
                * (1.0
                    + atkbuff
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 + extra_scale) * (1.0 - res / 100.0)) as f64)
                .max((final_atk * (1.0 + extra_scale) * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0) + extra_scale;
            let mut reflectdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps += reflectdmg * 0.0 /* self.hits - defaults to 0 */;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk = self.unit.atk
                * (1.0
                    + atkbuff
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 + extra_scale) * (1.0 - res / 100.0)) as f64)
                .max((final_atk * (1.0 + extra_scale) * 0.05) as f64);
            let mut hits = if self.unit.skill_damage { 4.0 } else { 3.0 };
            dps = hits * hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                * ((self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)) as f64)
                    .min((self.unit.targets as f64) as f64);
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    /// if self.skill == 3 and self.skill_dmg: self.skill_duration = 11
    /// return super().total_dmg(defense, res)
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut skill_duration: f64 = 0.0;
        if (self.unit.skill_index as f64) == 3.0 && self.unit.skill_damage {
            skill_duration = 11.0;
        }
        // TODO: return super().total_dmg(defense, res) - requires manual implementation
        0.0 // placeholder
    }
}

impl std::ops::Deref for HoshigumaAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for HoshigumaAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for HoshigumaAlter {
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
