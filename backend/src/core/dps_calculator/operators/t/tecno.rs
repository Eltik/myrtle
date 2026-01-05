//! DPS calculations for Tecno
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Tecno operator implementation
pub struct Tecno {
    pub unit: OperatorUnit,
}

impl Tecno {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Tecno operator
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
    /// atkbuff = self.skill_params[0] if self.skill == 1 else 0
    /// if self.trait_dmg and self.module == 2 and self.module_lvl == 3: atkbuff += 0.15
    /// aspd = self.skill_params[0] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1 - res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// final_drone = self.drone_atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// drone_hitdmg = np.fmax(final_drone * (1 - res/100), final_drone * 0.05)
    /// aspd_correction = 4 + self.module_lvl if self.module == 2 else 0
    /// drone_dps = drone_hitdmg/self.drone_atk_interval * (self.attack_speed + aspd - aspd_correction)/100
    /// if self.trait_dmg:
    /// drones = self.talent1_params[0] if self.talent_dmg else 1
    /// dps += drones * drone_dps
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

        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if self.unit.trait_damage
            && (self.unit.module_index as f64) == 2.0
            && (self.unit.module_level as f64) == 3.0
        {
            atkbuff += 0.15;
        }
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
        let mut dps =
            hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        let mut final_drone =
            self.unit.drone_atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut drone_hitdmg =
            ((final_drone * (1.0 - res / 100.0)) as f64).max((final_drone * 0.05) as f64);
        let mut aspd_correction = if ((self.unit.module_index as f64) as f64) == 2.0 {
            4.0 + ((self.unit.module_level as f64) as f64)
        } else {
            0.0
        };
        let mut drone_dps = drone_hitdmg / (self.unit.drone_atk_interval as f64)
            * (self.unit.attack_speed + aspd - aspd_correction)
            / 100.0;
        if self.unit.trait_damage {
            let mut drones = if self.unit.talent_damage {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            dps += drones * drone_dps;
        }
        return dps;
    }
}

impl std::ops::Deref for Tecno {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Tecno {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
