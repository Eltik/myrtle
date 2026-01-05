//! DPS calculations for Flametail
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Flametail operator implementation
pub struct Flametail {
    pub unit: OperatorUnit,
}

impl Flametail {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Flametail operator
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
    /// atkbuff = 0.08 if self.module == 1 and self.module_dmg else 0
    /// cdmg = 1
    /// if self.module == 1 and self.module_lvl > 1: cdmg = 1.2 if self.module_lvl == 3 else 1.15
    /// critrate = 0
    /// atk_interval = 1.05 * 0.7 if self.skill == 3 else self.atk_interval
    /// dodge = self.talent2_params[0] if self.elite == 2 else 0
    /// if self.skill == 3:
    /// dodge = 1-((1-dodge)*(1-self.skill_params[2]))
    /// if self.hits > 0:
    /// dodgerate = dodge * self.hits
    /// atkrate = 1/atk_interval * self.attack_speed/100
    /// critrate = min(1, dodgerate/atkrate)
    ///
    /// if self.skill < 2:
    /// final_atk = self.atk * (1+ self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05) * 2 * min(2, self.targets)
    /// avghit = critrate * critdmg + (1 - critrate) * hitdmg
    /// dps = avghit/atk_interval * self.attack_speed/100
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1+ self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05) * 2 * min(3, self.targets)
    /// avghit = critrate * critdmg + (1 - critrate) * hitdmg
    /// dps = avghit/atk_interval * self.attack_speed/100
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
        let mut critdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        atkbuff = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            0.08
        } else {
            0.0
        };
        cdmg = 1.0;
        if (self.unit.module_index as f64) == 1.0 && (self.unit.module_level as f64) > 1.0 {
            cdmg = if ((self.unit.module_level as f64) as f64) == 3.0 {
                1.2
            } else {
                1.15
            };
        }
        let mut critrate = 0.0;
        atk_interval = if ((self.unit.skill_index as f64) as f64) == 3.0 {
            1.05 * 0.7
        } else {
            (self.unit.attack_interval as f64)
        };
        let mut dodge = if ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 3.0 {
            dodge = 1.0
                - ((1.0 - dodge)
                    * (1.0 - self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)));
        }
        if 0.0 /* self.hits - defaults to 0 */ > 0.0 {
            let mut dodgerate = dodge * 0.0 /* self.hits - defaults to 0 */;
            let mut atkrate = 1.0 / atk_interval * self.unit.attack_speed / 100.0;
            critrate = ((1) as f64).min((dodgerate / atkrate) as f64);
        }
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64)
                * 2.0
                * ((2) as f64).min((self.unit.targets as f64) as f64);
            avghit = critrate * critdmg + (1.0 - critrate) * hitdmg;
            dps = avghit / atk_interval * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64)
                * 2.0
                * ((3) as f64).min((self.unit.targets as f64) as f64);
            avghit = critrate * critdmg + (1.0 - critrate) * hitdmg;
            dps = avghit / atk_interval * self.unit.attack_speed / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Flametail {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Flametail {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
