//! DPS calculations for SilverAsh
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// SilverAsh operator implementation
pub struct SilverAsh {
    pub unit: OperatorUnit,
}

impl SilverAsh {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new SilverAsh operator
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
    /// atk_scale = 1
    /// if not self.trait_dmg and self.skill != 3:
    /// atk_scale = 0.8
    /// atkbuff = self.talent1_params[0]
    ///
    /// if self.module == 1:
    /// if self.module_lvl == 2:
    /// if self.talent_dmg and self.module_dmg: atk_scale *= 1.1
    /// if self.module_lvl == 3:
    /// if self.talent_dmg and self.module_dmg: atk_scale *= 1.15
    ///
    /// bonus = 0.1 if self.module == 1 else 0
    ///
    /// ####the actual skills
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk* atk_scale * skill_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1)
    /// dps = avgphys/(self.atk_interval/(self.attack_speed/100)) + bonusdmg/(self.atk_interval/(self.attack_speed/100))
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) + bonusdmg/(self.atk_interval/(self.attack_speed/100))
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[1]
    /// targets = self.skill_params[2]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) * min(self.targets, targets) + bonusdmg/(self.atk_interval/(self.attack_speed/100)) * min(self.targets,targets)
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

        let mut atk_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut bonusdmg: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        atk_scale = 1.0;
        if !self.unit.trait_damage && (self.unit.skill_index as f64) != 3.0 {
            atk_scale = 0.8;
        }
        atkbuff = self.unit.talent1_parameters[0];
        if (self.unit.module_index as f64) == 1.0 {
            if (self.unit.module_level as f64) == 2.0 {
                // UNTRANSLATED: if self.talent_dmg and self.module_dmg: atk_scale *= 1.1
            }
            if (self.unit.module_level as f64) == 3.0 {
                // UNTRANSLATED: if self.talent_dmg and self.module_dmg: atk_scale *= 1.15
            }
        }
        let mut bonus = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        // ###the actual skills
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters[0]
            } else {
                1.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1.0);
            dps = avgphys / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                + bonusdmg
                    / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                + bonusdmg
                    / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters[1];
            let mut targets = self.unit.skill_parameters[2];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * ((self.unit.targets as f64) as f64).min((targets) as f64)
                + bonusdmg
                    / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                    * ((self.unit.targets as f64) as f64).min((targets) as f64);
        }
        dps
    }
}

impl std::ops::Deref for SilverAsh {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for SilverAsh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
