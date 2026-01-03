//! DPS calculations for Firewhistle
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Firewhistle operator implementation
pub struct Firewhistle {
    pub unit: OperatorUnit,
}

impl Firewhistle {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Firewhistle operator
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
    /// atkbuff = self.talent1_params[0] if self.talent_dmg else 0
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[2]
    /// fire_scale = self.skill_params[1] * self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// hitdmgskill = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05)
    /// hitdmgarts = np.fmax(final_atk * atk_scale * fire_scale * (1-res/100), final_atk * 0.05)
    /// avgdmg = 3/4 * self.targets * hitdmg + 1/4 * hitdmgskill * self.targets + hitdmgarts / 4
    /// if self.skill == 0: avgdmg = hitdmg
    /// dps = avgdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// hitdmgarts = np.fmax(final_atk * atk_scale * skill_scale * (1-res/100), final_atk * atk_scale * skill_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + hitdmgarts
    /// dps = dps * self.targets
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

        let mut dps: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        atkbuff = if self.unit.talent_damage {
            self.unit.talent1_parameters[0]
        } else {
            0.0
        };
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters[2];
            let mut fire_scale = self.unit.skill_parameters[1] * self.unit.skill_parameters[0];
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut hitdmgskill = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            hitdmgarts = ((final_atk * atk_scale * fire_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64);
            avgdmg = 3.0 / 4.0 * (self.unit.targets as f64) * hitdmg
                + 1.0 / 4.0 * hitdmgskill * (self.unit.targets as f64)
                + hitdmgarts / 4.0;
            if (self.unit.skill_index as f64) == 0.0 {
                avgdmg = hitdmg;
            }
            dps = avgdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[1];
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            hitdmgarts = ((final_atk * atk_scale * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                + hitdmgarts;
            dps *= (self.unit.targets as f64);
        }
        dps
    }
}

impl std::ops::Deref for Firewhistle {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Firewhistle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
