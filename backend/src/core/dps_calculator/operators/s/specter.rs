//! DPS calculations for Specter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Specter operator implementation
pub struct Specter {
    pub unit: OperatorUnit,
}

impl Specter {
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
    )] = &[("module", "vsBlocked", false, &[], &[1], 0, 0)];

    /// Creates a new Specter operator
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
    /// atk_scale = 1.1 if self.module_dmg and self.module == 1 else 1
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * min(self.skill,1)) + self.buff_atk_flat
    /// dmgbuff = 1 if self.module_lvl < 2 else 1.03
    /// if self.module_lvl == 3: dmgbuff = 1.05
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)*(dmgbuff)
    /// targets = 3 if self.elite == 2 else 2
    /// dps = hitdmg / self.atk_interval * self.attack_speed/100 * min(self.targets, targets)
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

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut atk_scale =
            if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 1.0 {
                1.1
            } else {
                1.0
            };
        let mut final_atk = self.unit.atk
            * (1.0
                + self.unit.buff_atk
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                    * ((self.unit.skill_index as f64) as f64).min((1) as f64))
            + self.unit.buff_atk_flat;
        let mut dmgbuff = if ((self.unit.module_level as f64) as f64) < 2.0 {
            1.0
        } else {
            1.03
        };
        if (self.unit.module_level as f64) == 3.0 {
            dmgbuff = 1.05;
        }
        let mut hitdmg = ((final_atk * atk_scale - defense) as f64)
            .max((final_atk * atk_scale * 0.05) as f64)
            * (dmgbuff);
        let mut targets = if ((self.unit.elite as f64) as f64) == 2.0 {
            3.0
        } else {
            2.0
        };
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
            * ((self.unit.targets as f64) as f64).min((targets) as f64);
        return dps;
    }
}

impl std::ops::Deref for Specter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Specter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Specter {
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
