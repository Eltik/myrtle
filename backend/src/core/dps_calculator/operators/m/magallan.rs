//! DPS calculations for Magallan
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Magallan operator implementation
pub struct Magallan {
    pub unit: OperatorUnit,
}

impl Magallan {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("trait", "noDrones", true, &[1], &[], 0, 0)];

    /// Creates a new Magallan operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let mut unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        // Apply init-time modifications from Python __init__
        if unit.module_index == 2 && unit.module_level == 3 && unit.skill_index == 2 {
            unit.drone_atk += 40.0;
        }
        if unit.module_index == 2
            && unit.module_level == 3
            && (unit.skill_index == 0 || unit.skill_index == 3)
        {
            unit.drone_atk += 50.0;
        }

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// drones = 2 if self.talent_dmg else 1
    /// if not self.trait_dmg: drones = 0
    /// bonusaspd = 3 if self.module == 2 and self.module_lvl == 3 else 0
    ///
    /// if self.skill == 1:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// aspd = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// final_drone = self.drone_atk * (1 + self.buff_atk) + self.buff_atk_flat
    ///
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmgdrone = np.fmax(final_drone * (1-res/100), final_drone * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100 + hitdmgdrone/self.drone_atk_interval* (self.attack_speed+aspd+bonusaspd)/100 * drones * self.targets
    /// if self.skill in [0,3]:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]*self.skill/3) + self.buff_atk_flat
    /// final_drone = self.drone_atk * (1 + self.buff_atk + self.skill_params[0]*self.skill/3) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmgdrone = np.fmax(final_drone - defense, final_drone * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + hitdmgdrone/self.drone_atk_interval* (self.attack_speed+bonusaspd)/100 * drones * self.targets
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

        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_drone: f64 = 0.0;
        let mut hitdmgdrone: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;

        let mut drones = if self.unit.talent_damage { 2.0 } else { 1.0 };
        if !self.unit.trait_damage {
            drones = 0.0;
        }
        let mut bonusaspd = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) == 3.0
        {
            3.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            aspd = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            final_drone =
                self.unit.drone_atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            hitdmgdrone =
                ((final_drone * (1.0 - res / 100.0)) as f64).max((final_drone * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                + hitdmgdrone / (self.unit.drone_atk_interval as f64)
                    * (self.unit.attack_speed + aspd + bonusaspd)
                    / 100.0
                    * drones
                    * (self.unit.targets as f64);
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 3.0)
                + self.unit.buff_atk_flat;
            final_drone = self.unit.drone_atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 3.0)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            hitdmgdrone = ((final_drone - defense) as f64).max((final_drone * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                + hitdmgdrone / (self.unit.drone_atk_interval as f64)
                    * (self.unit.attack_speed + bonusaspd)
                    / 100.0
                    * drones
                    * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Magallan {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Magallan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Magallan {
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
