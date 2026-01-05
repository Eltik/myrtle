//! DPS calculations for Goldenglow
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Goldenglow operator implementation
pub struct Goldenglow {
    pub unit: OperatorUnit,
}

impl Goldenglow {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Goldenglow operator
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
    /// newres = np.fmax(res-self.talent2_params[0],0)
    /// drone_dmg = 1.2 if self.module == 2 else 1.1
    /// drone_explosion = self.talent1_params[1] if self.elite > 0 else 0
    /// explosion_prob = 0.1 if self.elite > 0 else 0
    /// aspd = 0
    /// drones = 2
    /// if not self.trait_dmg:
    /// drone_dmg = 0.35 if self.module == 1 else 0.2
    /// atkbuff = self.skill_params[0] * min(self.skill,1)
    /// if self.skill == 1:
    /// aspd += self.skill_params[1]
    /// if self.skill == 3:
    /// drones = 3
    /// final_atk = self.atk * (1+atkbuff+self.buff_atk) + self.buff_atk_flat
    /// drone_atk = drone_dmg * final_atk
    /// drone_explosion = final_atk * drone_explosion * self.targets
    /// dmgperinterval = final_atk*(3-drones) + drones * drone_atk * (1-explosion_prob) + drones * drone_explosion * explosion_prob
    /// if self.skill == 0: dmgperinterval = final_atk + drone_atk
    /// hitdmgarts = np.fmax(dmgperinterval *(1-newres/100), dmgperinterval * 0.05)
    /// dps = hitdmgarts/self.atk_interval*(self.attack_speed+aspd)/100
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

        let mut hitdmgarts: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut dps: f64 = 0.0;

        newres = ((res - self.unit.talent2_parameters.first().copied().unwrap_or(0.0)) as f64)
            .max((0) as f64);
        let mut drone_dmg = if ((self.unit.module_index as f64) as f64) == 2.0 {
            1.2
        } else {
            1.1
        };
        let mut drone_explosion = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut explosion_prob = if ((self.unit.elite as f64) as f64) > 0.0 {
            0.1
        } else {
            0.0
        };
        aspd = 0.0;
        let mut drones = 2.0;
        if !self.unit.trait_damage {
            drone_dmg = if ((self.unit.module_index as f64) as f64) == 1.0 {
                0.35
            } else {
                0.2
            };
        }
        atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            * ((self.unit.skill_index as f64) as f64).min((1) as f64);
        if (self.unit.skill_index as f64) == 1.0 {
            aspd += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            drones = 3.0;
        }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut drone_atk = drone_dmg * final_atk;
        drone_explosion = final_atk * drone_explosion * (self.unit.targets as f64);
        let mut dmgperinterval = final_atk * (3.0 - drones)
            + drones * drone_atk * (1.0 - explosion_prob)
            + drones * drone_explosion * explosion_prob;
        if (self.unit.skill_index as f64) == 0.0 {
            dmgperinterval = final_atk + drone_atk;
        }
        hitdmgarts =
            ((dmgperinterval * (1.0 - newres / 100.0)) as f64).max((dmgperinterval * 0.05) as f64);
        dps = hitdmgarts / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Goldenglow {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Goldenglow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
