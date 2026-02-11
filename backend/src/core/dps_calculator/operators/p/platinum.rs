//! DPS calculations for Platinum
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Platinum operator implementation
pub struct Platinum {
    pub unit: OperatorUnit,
}

impl Platinum {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("module", "aerial target", false, &[], &[1], 0, 0)];

    /// Creates a new Platinum operator
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
    /// aspd = -20 if self.skill == 2 else 0
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// final_atk = self.atk * (1 + max(self.skill_params) * min(self.skill, 1) + self.buff_atk) + self.buff_atk_flat
    /// if self.elite > 0:
    /// extra_scale = self.talent1_params[3] - 1
    /// atk_cycle = self.atk_interval /(self.attack_speed + aspd) * 100
    /// charge_time = max(atk_cycle - 1, 0)
    /// weight = min(1, charge_time / 1.5)
    /// atk_scale *= 1 + weight * extra_scale
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
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

        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            -20.0
        } else {
            0.0
        };
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        final_atk = self.unit.atk
            * (1.0
                + self
                    .unit
                    .skill_parameters
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max)
                    * ((self.unit.skill_index as f64) as f64).min((1) as f64)
                + self.unit.buff_atk)
            + self.unit.buff_atk_flat;
        if (self.unit.elite as f64) > 0.0 {
            let mut extra_scale = self.unit.talent1_parameters.get(3).copied().unwrap_or(0.0) - 1.0;
            let mut atk_cycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed + aspd) * 100.0;
            let mut charge_time = ((atk_cycle - 1.0) as f64).max((0) as f64);
            let mut weight = ((1) as f64).min((charge_time / 1.5) as f64);
            atk_scale *= 1.0 + weight * extra_scale;
        }
        hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Platinum {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Platinum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Platinum {
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
