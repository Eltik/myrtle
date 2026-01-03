//! DPS calculations for Siege
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Siege operator implementation
pub struct Siege {
    pub unit: OperatorUnit,
}

impl Siege {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Siege operator
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
    /// atkbuff = 0.08 if self.module == 1 and self.module_dmg else 0
    /// atkbuff += self.talent1_params[0]
    /// if self.module == 1 and self.module_lvl > 1: atkbuff += 0.02 + 0.02 * self.module_lvl
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// if self.skill < 2:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * self.targets
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[2] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    /// if self.skill == 3:
    /// atk_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/2.05 * self.attack_speed/100
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

        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        atkbuff = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            0.08
        } else {
            0.0
        };
        atkbuff += self.unit.talent1_parameters[0];
        // UNTRANSLATED: if self.module == 1 and self.module_lvl > 1: atkbuff += 0.02 + 0.02 * self.module_lvl
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[0];
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * (self.unit.targets as f64);
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                if self.unit.skill_parameters[2] > 1.0 {
                    avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_scale = self.unit.skill_parameters[0];
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / 2.05 * self.unit.attack_speed / 100.0;
        }
        dps
    }
}

impl std::ops::Deref for Siege {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Siege {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
