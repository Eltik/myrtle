//! DPS calculations for TexasAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// TexasAlter operator implementation
pub struct TexasAlter {
    pub unit: OperatorUnit,
}

impl TexasAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new TexasAlter operator
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
    ///
    /// aspd = self.talent2_params[0] if self.elite == 2 and self.talent2_dmg else 0
    /// atkbuff = self.talent1_params[0] if self.elite > 0 else 0
    /// atkbuff += self.skill_params[0] if self.skill != 3 else 0
    /// if self.skill == 0: atkbuff = 0
    /// if self.module == 2 and not self.module_dmg: atkbuff += 0.1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    ///
    /// if self.skill < 2:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// artsdmg = self.skill_params[2] if self.skill == 1 else 0
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 + np.fmax(artsdmg *(1-res/100), artsdmg * 0.05)
    /// if self.skill == 2:
    /// newres = res *(1+self.skill_params[1])
    /// hitdmgarts = np.fmax(final_atk *(1-newres/100), final_atk * 0.05)
    /// dps = 2 * hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 3:
    /// skillscale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * skillscale *(1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// dps += hitdmgarts * min(self.targets, self.skill_params[2])
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
        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut newres: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;

        aspd = if ((self.unit.elite as f64) as f64) == 2.0 && self.unit.talent2_damage {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        atkbuff = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        atkbuff += if ((self.unit.skill_index as f64) as f64) != 3.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            atkbuff = 0.0;
        }
        if (self.unit.module_index as f64) == 2.0 && !self.unit.module_damage {
            atkbuff += 0.1;
        }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut artsdmg = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                + ((artsdmg * (1.0 - res / 100.0)) as f64).max((artsdmg * 0.05) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            newres = res * (1.0 + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0));
            hitdmgarts =
                ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = 2.0 * hitdmgarts / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut skillscale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * skillscale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
            dps += hitdmgarts
                * ((self.unit.targets as f64) as f64)
                    .min((self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for TexasAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for TexasAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for TexasAlter {
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
