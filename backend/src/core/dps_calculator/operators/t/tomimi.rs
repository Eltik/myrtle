//! DPS calculations for Tomimi
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Tomimi operator implementation
pub struct Tomimi {
    pub unit: OperatorUnit,
}

impl Tomimi {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Tomimi operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atkbuff = self.talent1_params[0] if self.skill > 0 else 0
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// if self.skill == 0:
    /// hitdmg = np.fmax(final_atk * (1 - res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 1:
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + self.skill_params[0]) / 100
    /// if self.skill == 2:
    /// crate = self.skill_params[2]
    /// atk_scale = self.skill_params[1]
    /// critdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// avgnormal = (1-crate) * hitdmg
    /// avgstun = crate / 3 * hitdmg
    /// avgcrit = crate / 3 * critdmg
    /// avgaoe = crate / 3 * hitdmg * self.targets
    /// dps = (avgnormal + avgstun + avgcrit + avgaoe)/self.atk_interval * self.attack_speed/100
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

        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut critdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        if (self.unit.skill_index as f64) == 0.0 {
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            dps = hitdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut crit_rate = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            atk_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            critdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut avgnormal = (1.0 - crit_rate) * hitdmg;
            let mut avgstun = crit_rate / 3.0 * hitdmg;
            let mut avgcrit = crit_rate / 3.0 * critdmg;
            let mut avgaoe = crit_rate / 3.0 * hitdmg * (self.unit.targets as f64);
            dps = (avgnormal + avgstun + avgcrit + avgaoe) / (self.unit.attack_interval as f64)
                * self.unit.attack_speed
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Tomimi {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Tomimi {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Tomimi {
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
