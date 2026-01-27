//! DPS calculations for Ines
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Ines operator implementation
pub struct Ines {
    pub unit: OperatorUnit,
}

impl Ines {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("skill", "maxSteal", false, &[], &[], 0, 0)];

    /// Creates a new Ines operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// stolen_atk = 0 if self.elite < 1 else self.talent1_params[0]
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat + stolen_atk
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skillhitdmg = np.fmax(final_atk * skill_scale * (1 - res/100), final_atk * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + skillhitdmg * min(1, 3 / ((sp_cost+1) * self.atk_interval/self.attack_speed*100))
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[1]
    /// aspd = self.skill_params[3] if self.skill_dmg else self.skill_params[2]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat + stolen_atk
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill in [0,3]:
    /// atkbuff = self.skill_params[1] if self.skill == 3 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat + stolen_atk
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
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
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        let mut stolen_atk = if ((self.unit.elite as f64) as f64) < 1.0 {
            0.0
        } else {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        };
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat + stolen_atk;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                + skillhitdmg
                    * ((1) as f64).min(
                        (3.0 / ((sp_cost + 1.0) * (self.unit.attack_interval as f64)
                            / self.unit.attack_speed
                            * 100.0)) as f64,
                    );
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            aspd = if self.unit.skill_damage {
                self.unit.skill_parameters.get(3).copied().unwrap_or(0.0)
            } else {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff)
                + self.unit.buff_atk_flat
                + stolen_atk;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff = if ((self.unit.skill_index as f64) as f64) == 3.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff)
                + self.unit.buff_atk_flat
                + stolen_atk;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Ines {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ines {
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
