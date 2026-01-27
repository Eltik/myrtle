//! DPS calculations for Pudding
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Pudding operator implementation
pub struct Pudding {
    pub unit: OperatorUnit,
}

impl Pudding {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[];

    /// Creates a new Pudding operator
    #[allow(unused_parens)]
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
    /// atkbuff = self.talent1_params[0]
    /// targetscaling = [0,1,2,3,4] if self.module == 2 else [0, 1, 1.85, 1.85+0.85**2, 1.85+0.85**2+0.85**3]
    /// if self.elite < 2 and not self.skill == 2: targetscaling[4] = targetscaling[3]
    /// targets = min(4, self.targets)
    /// if self.skill == 1:
    /// aspd = self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * targetscaling[targets]
    /// if self.skill in [0,2]:
    /// atkbuff += self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.targets > 1: dps = hitdmg/self.atk_interval * self.attack_speed/100  * targetscaling[4]
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

        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;

        atkbuff = self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0);
        let mut targetscaling = if ((self.unit.module_index as f64) as f64) == 2.0 {
            [0.0, 1.0, 2.0, 3.0, 4.0]
        } else {
            [
                0.0,
                1.0,
                1.85,
                1.85 + (0.85 as f64).powf(2 as f64),
                1.85 + (0.85 as f64).powf(2 as f64) + (0.85 as f64).powf(3 as f64),
            ]
        };
        // TODO: not translated: if self.elite < 2 and not self.skill == 2: targetscaling[4] = targetscaling[3]
        let mut targets = ((4) as f64).min((self.unit.targets as f64) as f64);
        if (self.unit.skill_index as f64) == 1.0 {
            aspd = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * targetscaling[(targets) as usize];
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff += if ((self.unit.skill_index as f64) as f64) == 2.0 {
                self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if (self.unit.targets as f64) > 1.0 {
                dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                    * targetscaling[4];
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Pudding {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Pudding {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Pudding {
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
