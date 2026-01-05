//! DPS calculations for Kjera
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Kjera operator implementation
pub struct Kjera {
    pub unit: OperatorUnit,
}

impl Kjera {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Kjera operator
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
    /// drone_dmg = 1.2 if self.module == 2 else 1.1
    /// if not self.trait_dmg: drone_dmg = 0.2
    /// atkbuff = 0
    /// if self.elite > 0: atkbuff += self.talent1_params[2] if self.talent_dmg else self.talent1_params[0]
    ///
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + self.skill_params[0] * min(self.skill, 1)) + self.buff_atk_flat
    /// drone_atk = drone_dmg * final_atk
    /// dmgperinterval = final_atk + drone_atk * self.skill
    /// if self.skill < 2:
    /// hitdmgarts = np.fmax(dmgperinterval *(1-res/100), dmgperinterval * 0.05)
    /// dps = hitdmgarts/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// res2 = np.fmax(0,res-15)
    /// hitdmgarts = np.fmax(dmgperinterval *(1-res/100), dmgperinterval * 0.05)
    /// hitdmgfreeze = np.fmax(dmgperinterval *(1-res2/100), dmgperinterval * 0.05)
    /// damage = hitdmgfreeze * self.freezeRate + hitdmgarts * (1 - self.freezeRate)
    /// dps = damage/self.atk_interval * self.attack_speed/100
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
        let mut hitdmgarts: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        let mut drone_dmg = if ((self.unit.module_index as f64) as f64) == 2.0 {
            1.2
        } else {
            1.1
        };
        if !self.unit.trait_damage {
            drone_dmg = 0.2;
        }
        atkbuff = 0.0;
        if (self.unit.elite as f64) > 0.0 {
            atkbuff += if self.unit.talent_damage {
                self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            };
        }
        final_atk = self.unit.atk
            * (1.0
                + atkbuff
                + self.unit.buff_atk
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                    * ((self.unit.skill_index as f64) as f64).min((1) as f64))
            + self.unit.buff_atk_flat;
        let mut drone_atk = drone_dmg * final_atk;
        let mut dmgperinterval = final_atk + drone_atk * (self.unit.skill_index as f64);
        if (self.unit.skill_index as f64) < 2.0 {
            hitdmgarts =
                ((dmgperinterval * (1.0 - res / 100.0)) as f64).max((dmgperinterval * 0.05) as f64);
            dps = hitdmgarts / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut res2 = ((0) as f64).max((res - 15.0) as f64);
            hitdmgarts =
                ((dmgperinterval * (1.0 - res / 100.0)) as f64).max((dmgperinterval * 0.05) as f64);
            let mut hitdmgfreeze = ((dmgperinterval * (1.0 - res2 / 100.0)) as f64)
                .max((dmgperinterval * 0.05) as f64);
            let mut damage = hitdmgfreeze * 0.0 /* self.freezeRate - needs manual implementation */ + hitdmgarts * (1.0 - 0.0 /* self.freezeRate - needs manual implementation */);
            dps = damage / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Kjera {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Kjera {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
