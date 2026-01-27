//! DPS calculations for YahataUmiri
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// YahataUmiri operator implementation
pub struct YahataUmiri {
    pub unit: OperatorUnit,
}

impl YahataUmiri {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "vsSlow", false, &[], &[], 0, 0),
        ("skill", "Fever", false, &[1], &[], 0, 0),
    ];

    /// Creates a new YahataUmiri operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// dmg = self.talent2_params[0] if self.talent_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.skill == 0:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100 * self.targets
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skillhit = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * 5
    /// if self.skill_dmg:
    /// dps = skillhit / self.atk_interval * self.attack_speed / 100 * self.targets
    /// else:
    /// dpsnorm = hitdmg / self.atk_interval * self.attack_speed / 100
    /// dpsskill = skillhit / 2.1
    /// interval = self.atk_interval / self.attack_speed * 100
    /// dps = (dpsnorm * self.skill_cost * interval + dpsskill * 2.1)/(self.skill_cost * interval + 2.1) * self.targets
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// dps = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * self.targets
    /// return dps * dmg
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

        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        let mut dmg = if self.unit.talent_damage {
            self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 0.0 {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut skillhit = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * 5.0;
            if self.unit.skill_damage {
                dps = skillhit / (self.unit.attack_interval as f64) * self.unit.attack_speed
                    / 100.0
                    * (self.unit.targets as f64);
            } else {
                let mut dpsnorm =
                    hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
                let mut dpsskill = skillhit / 2.1;
                let mut interval =
                    (self.unit.attack_interval as f64) / self.unit.attack_speed * 100.0;
                dps = (dpsnorm * (self.unit.skill_cost as f64) * interval + dpsskill * 2.1)
                    / ((self.unit.skill_cost as f64) * interval + 2.1)
                    * (self.unit.targets as f64);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            dps = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * (self.unit.targets as f64);
        }
        return dps * dmg;
    }
}

impl std::ops::Deref for YahataUmiri {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for YahataUmiri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for YahataUmiri {
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
