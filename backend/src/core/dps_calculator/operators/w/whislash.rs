//! DPS calculations for Whislash
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Whislash operator implementation
pub struct Whislash {
    pub unit: OperatorUnit,
}

impl Whislash {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Whislash operator
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
    /// atk_scale = 1
    /// if self.trait_dmg:
    /// atk_scale = 1.3 if self.module == 1 else 1.2
    /// talent_buff = self.talent1_params[0]
    /// atkbuff = self.skill_params[1] if self.skill == 2 else 0
    /// aspd = talent_buff * self.skill_params[0] if self.skill == 2 else 0.5 * talent_buff * self.skill_params[0] * self.skill
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// targets = 3 if self.skill == 2 else 1
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100 * min(targets, self.targets)
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut atk_scale = 1.0;
        if self.unit.trait_damage {
            atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
                1.3
            } else {
                1.2
            };
        }
        let mut talent_buff = self.unit.talent1_parameters[0];
        let mut atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters[1]
        } else {
            0.0
        };
        let mut aspd = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            talent_buff * self.unit.skill_parameters[0]
        } else {
            0.5 * talent_buff
                * self.unit.skill_parameters[0]
                * ((self.unit.skill_index as f64) as f64)
        };
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        let mut targets = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            3.0
        } else {
            1.0
        };
        
        hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
            / 100.0
            * ((targets) as f64).min((self.unit.targets as f64) as f64)
    }
}

impl std::ops::Deref for Whislash {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Whislash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
