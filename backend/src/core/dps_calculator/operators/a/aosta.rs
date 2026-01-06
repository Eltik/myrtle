//! DPS calculations for Aosta
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Aosta operator implementation
pub struct Aosta {
    pub unit: OperatorUnit,
}

impl Aosta {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[
        ("trait", "distant", true, &[], &[], 0, 0),
        ("talent", "blockedTarget", true, &[], &[], 1, 0),
    ];

    /// Creates a new Aosta operator
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
    /// atk_scale = 1.5 if self.trait_dmg else 1
    /// if self.trait_dmg and self.module == 1: atk_scale = 1.6
    /// talent_scale = self.talent1_params[0] if self.elite > 0 and self.talent_dmg else 0
    /// talent_duration = self.talent1_params[1]
    /// aspd = self.skill_params[1] if self.skill == 1 else 0
    /// if self.skill < 2:
    /// final_atk = self.atk * (1 + self.skill_params[0] * self.skill + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * self.targets
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.skill_params[1] + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg / 3.45 * self.attack_speed / 100 * self.targets
    /// talent_scale *= 2
    /// active_ratio = min(1, talent_duration/ (self.atk_interval / (self.attack_speed+aspd) * 100))
    /// arts_dps = np.fmax(final_atk * talent_scale * (1-res/100), final_atk * talent_scale * 0.05) * active_ratio * self.targets
    /// dps += arts_dps
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

        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;

        atk_scale = if self.unit.trait_damage { 1.5 } else { 1.0 };
        if self.unit.trait_damage && (self.unit.module_index as f64) == 1.0 {
            atk_scale = 1.6;
        }
        let mut talent_scale = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage
        {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut talent_duration = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                    + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                    + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / 3.45 * self.unit.attack_speed / 100.0 * (self.unit.targets as f64);
            talent_scale *= 2.0;
        }
        let mut active_ratio = ((1) as f64).min(
            (talent_duration
                / ((self.unit.attack_interval as f64) / (self.unit.attack_speed + aspd) * 100.0))
                as f64,
        );
        let mut arts_dps = ((final_atk * talent_scale * (1.0 - res / 100.0)) as f64)
            .max((final_atk * talent_scale * 0.05) as f64)
            * active_ratio
            * (self.unit.targets as f64);
        dps += arts_dps;
        return dps;
    }
}

impl std::ops::Deref for Aosta {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Aosta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Aosta {
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
