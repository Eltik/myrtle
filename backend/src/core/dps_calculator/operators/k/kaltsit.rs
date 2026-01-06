//! DPS calculations for Kaltsit
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Kaltsit operator implementation
pub struct Kaltsit {
    pub unit: OperatorUnit,
}

impl Kaltsit {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("talent", "NotInHealRange", true, &[], &[2], 0, 2)];

    /// Creates a new Kaltsit operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let mut unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        // Apply init-time modifications from Python __init__
        if (unit.module_index == 2 || unit.module_index == 3) {
            unit.attack_speed -= 4.0 + (unit.module_level as f64);
        }

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// aspd = 0
    /// if self.module == 2 and self.talent_dmg:
    /// if self.module_lvl == 2: aspd = 12
    /// if self.module_lvl == 3: aspd = 20
    /// atkbuff = 0.25 * (self.module_lvl - 1) if self.module == 3 else 0
    ///
    /// if self.skill < 2:
    /// final_atk = self.drone_atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.drone_atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// final_atk = self.drone_atk * (1 + self.buff_atk + self.skill_params[1] + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.drone_atk_interval * (self.attack_speed+aspd)/100 * min(self.targets,3)
    /// if self.skill == 3:
    /// final_atk = self.drone_atk * (1 + self.buff_atk + self.skill_params[0] * 0.5 + atkbuff) + self.buff_atk_flat
    /// dps = final_atk/self.drone_atk_interval * (self.attack_speed+aspd)/100 * np.fmax(-defense, 1)
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
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;

        aspd = 0.0;
        if (self.unit.module_index as f64) == 2.0 && self.unit.talent_damage {
            if (self.unit.module_level as f64) == 2.0 {
                aspd = 12.0;
            }
            if (self.unit.module_level as f64) == 3.0 {
                aspd = 20.0;
            }
        }
        atkbuff = if ((self.unit.module_index as f64) as f64) == 3.0 {
            0.25 * (((self.unit.module_level as f64) as f64) - 1.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.drone_atk * (1.0 + self.unit.buff_atk + atkbuff)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.drone_atk_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.drone_atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                    + atkbuff)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.drone_atk_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk = self.unit.drone_atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0) * 0.5
                    + atkbuff)
                + self.unit.buff_atk_flat;
            dps = final_atk / (self.unit.drone_atk_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0
                * ((-defense) as f64).max((1) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Kaltsit {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Kaltsit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Kaltsit {
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
