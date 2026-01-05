//! DPS calculations for Rosmontis
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Rosmontis operator implementation
pub struct Rosmontis {
    pub unit: OperatorUnit,
}

impl Rosmontis {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 3];

    /// Creates a new Rosmontis operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
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
    /// bonushits = 2 if self.module == 1 else 1
    /// bonusart = 1 if self.module == 3 else 0
    /// defshred = self.talent1_params[0] if self.elite > 0 else 0
    /// newdef = np.fmax(0, defense - defshred)
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + self.talent2_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk  * 0.05)
    /// bonushitdmg = np.fmax(final_atk * 0.5 - newdef, final_atk * 0.5 * 0.05) * bonushits
    /// bonushitdmg += np.fmax(final_atk * (1-res/100), final_atk * 0.05) * bonusart
    /// skillhitdmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// avghit = ((sp_cost + 1) * (hitdmg + bonushitdmg) + skillhitdmg) / (sp_cost + 1)
    /// if self.skill == 0: avghit = hitdmg + bonushitdmg
    /// dps = avghit/self.atk_interval * self.attack_speed/100 * self.targets
    /// if self.skill == 2:
    /// bonushits += 2
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[1] + self.talent2_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// bonushitdmg = np.fmax(final_atk * 0.5 - newdef, final_atk * 0.5 * 0.05) * bonushits
    /// bonushitdmg += np.fmax(final_atk * (1-res/100), final_atk * 0.05) * bonusart
    /// dps = (hitdmg+ bonushitdmg)/3.15 * self.attack_speed/100 * self.targets
    /// if self.skill == 3:
    /// if self.skill_dmg:
    /// if self.shreds[0] < 1 and self.shreds[0] > 0:
    /// defense = defense / self.shreds[0]
    /// newdef= np.fmax(0, defense - 160)
    /// if self.shreds[0] < 1 and self.shreds[0] > 0:
    /// newdef *= self.shreds[0]
    /// newdef = np.fmax(0,newdef - defshred)
    /// else:
    /// newdef = np.fmax(0, defense- defshred)
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[1] + self.talent2_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// bonushitdmg = np.fmax(final_atk * 0.5 - newdef, final_atk * 0.5 * 0.05) * bonushits
    /// bonushitdmg += np.fmax(final_atk * (1-res/100), final_atk * 0.05) * bonusart
    /// dps = (hitdmg+ bonushitdmg)/1.05 * self.attack_speed/100 * self.targets * min(self.targets,2)
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
        let mut skill_scale: f64 = 0.0;
        let mut bonushitdmg: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut newdef: f64 = 0.0;

        let mut bonushits = if ((self.unit.module_index as f64) as f64) == 1.0 {
            2.0
        } else {
            1.0
        };
        let mut bonusart = if ((self.unit.module_index as f64) as f64) == 3.0 {
            1.0
        } else {
            0.0
        };
        defshred = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        newdef = ((0) as f64).max((defense - defshred) as f64);
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.talent2_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
            bonushitdmg = ((final_atk * 0.5 - newdef) as f64).max((final_atk * 0.5 * 0.05) as f64)
                * bonushits;
            bonushitdmg += ((final_atk * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * bonusart;
            let mut skillhitdmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            avghit = ((sp_cost + 1.0) * (hitdmg + bonushitdmg) + skillhitdmg) / (sp_cost + 1.0);
            if (self.unit.skill_index as f64) == 0.0 {
                avghit = hitdmg + bonushitdmg;
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            bonushits += 2.0;
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                    + self.unit.talent2_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
            bonushitdmg = ((final_atk * 0.5 - newdef) as f64).max((final_atk * 0.5 * 0.05) as f64)
                * bonushits;
            bonushitdmg += ((final_atk * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * bonusart;
            dps = (hitdmg + bonushitdmg) / 3.15 * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            if self.unit.skill_damage {
                if self.unit.shreds[0] < 1.0 && self.unit.shreds[0] > 0.0 {
                    let mut defense = defense / self.unit.shreds.first().copied().unwrap_or(0.0);
                }
                newdef = ((0) as f64).max((defense - 160.0) as f64);
                if self.unit.shreds[0] < 1.0 && self.unit.shreds[0] > 0.0 {
                    newdef *= self.unit.shreds.first().copied().unwrap_or(0.0);
                }
                newdef = ((0) as f64).max((newdef - defshred) as f64);
            } else {
                newdef = ((0) as f64).max((defense - defshred) as f64);
            }
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                    + self.unit.talent2_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
            bonushitdmg = ((final_atk * 0.5 - newdef) as f64).max((final_atk * 0.5 * 0.05) as f64)
                * bonushits;
            bonushitdmg += ((final_atk * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * bonusart;
            dps = (hitdmg + bonushitdmg) / 1.05 * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64)
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Rosmontis {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Rosmontis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Rosmontis {
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
