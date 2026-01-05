//! DPS calculations for SanktaMiksaparato
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// SanktaMiksaparato operator implementation
pub struct SanktaMiksaparato {
    pub unit: OperatorUnit,
}

impl SanktaMiksaparato {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new SanktaMiksaparato operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// aspd = self.talent1_params[1] * 3 if self.elite > 0 and self.talent_dmg else 0
    /// if self.skill < 2:
    /// final_atk = self.atk * (1+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk  - defense, final_atk  * 0.05)
    /// skillhitdmg = np.fmax(final_atk * self.skill_params[0] - defense, final_atk * self.skill_params[0] * 0.05) * 3
    /// avgphys = (self.skill_cost * hitdmg + skillhitdmg) / (self.skill_cost + 1)
    /// if self.skill == 0: avgphys = hitdmg
    /// dps = avgphys/(self.atk_interval/((self.attack_speed + aspd)/100))
    /// if self.skill == 2:
    /// final_atk = self.atk * (1+ self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 3:
    /// final_atk = self.atk * (1+ self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)    * min(self.targets,3)
    /// dps = hitdmg/self.atk_interval * 10 * (self.attack_speed+aspd)/100 if self.skill_dmg else hitdmg
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

        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avgphys: f64 = 0.0;

        aspd = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0) * 3.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut skillhitdmg = ((final_atk
                * self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                - defense) as f64)
                .max(
                    (final_atk * self.unit.skill_parameters.first().copied().unwrap_or(0.0) * 0.05)
                        as f64,
                )
                * 3.0;
            avgphys = ((self.unit.skill_cost as f64) * hitdmg + skillhitdmg)
                / ((self.unit.skill_cost as f64) + 1.0);
            if (self.unit.skill_index as f64) == 0.0 {
                avgphys = hitdmg;
            }
            dps = avgphys
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((3) as f64);
            dps = if self.unit.skill_damage {
                hitdmg / (self.unit.attack_interval as f64) * 10.0 * (self.unit.attack_speed + aspd)
                    / 100.0
            } else {
                hitdmg
            };
        }
        return dps;
    }
}

impl std::ops::Deref for SanktaMiksaparato {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for SanktaMiksaparato {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
