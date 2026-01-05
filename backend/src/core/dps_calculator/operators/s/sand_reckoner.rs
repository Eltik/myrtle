//! DPS calculations for SandReckoner
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// SandReckoner operator implementation
pub struct SandReckoner {
    pub unit: OperatorUnit,
}

impl SandReckoner {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new SandReckoner operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// drones = 2 if self.talent2_dmg else 1
    /// if not self.trait_dmg: drones = 0
    /// dmg = 1.1 + min(0.1, 0.1 * self.elite) if self.talent_dmg else 1
    /// aspd = self.skill_params[0] if self.skill == 1 else 0
    /// module_aspd = -1 + 3 * self.module_lvl if self.module == 1 and self.module_lvl > 1 else 0
    /// atkbuff = self.skill_params[1] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * dmg
    /// dps =  hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// final_atk_drone = self.drone_atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmgdrone = np.fmax(final_atk_drone *(1-res/100) , final_atk_drone * 0.05) * dmg
    /// dps += hitdmgdrone/self.drone_atk_interval * (self.attack_speed + aspd + module_aspd)/100 * drones
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

        let mut drones = if self.unit.talent2_damage { 2.0 } else { 1.0 };
        if !self.unit.trait_damage {
            drones = 0.0;
        }
        let mut dmg = if self.unit.talent_damage {
            1.1 + ((0.1) as f64).min((0.1 * ((self.unit.elite as f64) as f64)) as f64)
        } else {
            1.0
        };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut module_aspd = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            -1.0 + 3.0 * ((self.unit.module_level as f64) as f64)
        } else {
            0.0
        };
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64) * dmg;
        let mut dps =
            hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        let mut final_atk_drone =
            self.unit.drone_atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        let mut hitdmgdrone = ((final_atk_drone * (1.0 - res / 100.0)) as f64)
            .max((final_atk_drone * 0.05) as f64)
            * dmg;
        dps += hitdmgdrone / (self.unit.drone_atk_interval as f64)
            * (self.unit.attack_speed + aspd + module_aspd)
            / 100.0
            * drones;
        return dps;
    }
}

impl std::ops::Deref for SandReckoner {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for SandReckoner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for SandReckoner {
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
