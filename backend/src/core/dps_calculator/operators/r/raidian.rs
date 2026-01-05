//! DPS calculations for Raidian
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Raidian operator implementation
pub struct Raidian {
    pub unit: OperatorUnit,
}

impl Raidian {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[3, 2];

    /// Creates a new Raidian operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            6, // default_potential
            3, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// drones = 2 if self.talent_dmg else 1
    /// if not self.trait_dmg: drones = 0
    /// dmg = self.skill_params[6] if self.skill == 3 and drones > 0 else 1
    /// hits = 3 if self.skill == 2 else 1
    /// skill_attack = self.skill_params[0] if self.skill in [2,3] else 0
    /// final_atk = self.atk * (1 + self.buff_atk + skill_attack) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg
    /// final_drone = self.drone_atk * (1 + self.buff_atk + skill_attack) + self.buff_atk_flat + max(self.elite - 1,0) * self.talent2_params[0] * final_atk
    /// hitdmgdrone = np.fmax(final_drone - defense, final_drone * 0.05) * hits if self.skill in [1,2] else np.fmax(final_drone * (1-res/100), final_drone * 0.05) * dmg
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + hitdmgdrone/self.drone_atk_interval* (self.attack_speed)/100 * drones
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

        let mut drones = if self.unit.talent_damage { 2.0 } else { 1.0 };
        if !self.unit.trait_damage {
            drones = 0.0;
        }
        let mut dmg = if ((self.unit.skill_index as f64) as f64) == 3.0 && drones > 0.0 {
            self.unit.skill_parameters.get(6).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut hits = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            3.0
        } else {
            1.0
        };
        let mut skill_attack =
            if [2.0, 3.0].contains(&(((self.unit.skill_index as f64) as f64) as f64)) {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                0.0
            };
        let mut final_atk =
            self.unit.atk * (1.0 + self.unit.buff_atk + skill_attack) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64) * dmg;
        let mut final_drone = self.unit.drone_atk * (1.0 + self.unit.buff_atk + skill_attack)
            + self.unit.buff_atk_flat
            + (((self.unit.elite as f64) - 1.0) as f64).max((0) as f64)
                * self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
                * final_atk;
        let mut hitdmgdrone =
            if [1.0, 2.0].contains(&(((self.unit.skill_index as f64) as f64) as f64)) {
                ((final_drone - defense) as f64).max((final_drone * 0.05) as f64) * hits
            } else {
                ((final_drone * (1.0 - res / 100.0)) as f64).max((final_drone * 0.05) as f64) * dmg
            };
        let mut dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
            + hitdmgdrone / (self.unit.drone_atk_interval as f64) * (self.unit.attack_speed)
                / 100.0
                * drones;
        return dps;
    }
}

impl std::ops::Deref for Raidian {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Raidian {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
