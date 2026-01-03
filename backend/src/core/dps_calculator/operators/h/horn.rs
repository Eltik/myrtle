//! DPS calculations for Horn
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Horn operator implementation
pub struct Horn {
    pub unit: OperatorUnit,
}

impl Horn {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Horn operator
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
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// atkbuff = self.talent1_params[0]
    /// aspd = self.talent2_params[2] if self.talent2_dmg else 0
    /// if self.module == 2 and self.module_dmg: aspd += 10
    /// if self.module == 2 and self.module_lvl > 1:
    /// if self.module_lvl == 2: aspd += 5
    /// if self.module_lvl == 3: aspd += 8
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// sp_cost = self.skill_cost
    /// final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk* atk_scale * skill_scale * 0.05)
    /// sp_cost = sp_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/((self.attack_speed+aspd)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[3] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval*(self.attack_speed+aspd)/100 * self.targets
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// arts_scale = 0
    /// if self.skill_dmg:
    /// arts_scale = self.skill_params[1]
    /// final_atk = final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05)
    /// artsdmg = np.fmax(final_atk * atk_scale * arts_scale * (1-res/100), final_atk * atk_scale * arts_scale * 0.05)
    /// dps = (hitdmg+artsdmg)/self.atk_interval*(self.attack_speed+aspd)/100 * self.targets
    /// if self.skill == 3:
    /// atk_interval = self.atk_interval + self.skill_params[1]
    /// atkbuff += self.skill_params[0]
    /// if self.skill_dmg: atkbuff += self.skill_params[0]
    /// final_atk = final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/atk_interval*(self.attack_speed+aspd)/100 * self.targets
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

        let mut atk_interval: f64 = 0.0;
        let mut artsdmg: f64 = 0.0;
        let mut arts_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        atkbuff = self.unit.talent1_parameters[0];
        aspd = if self.unit.talent2_damage {
            self.unit.talent2_parameters[2]
        } else {
            0.0
        };
        // UNTRANSLATED: if self.module == 2 and self.module_dmg: aspd += 10
        if (self.unit.module_index as f64) == 2.0 && (self.unit.module_level as f64) > 1.0 {
            // UNTRANSLATED: if self.module_lvl == 2: aspd += 5
            // UNTRANSLATED: if self.module_lvl == 3: aspd += 8
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters[0]
            } else {
                1.0
            };
            sp_cost = (self.unit.skill_cost as f64);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            skilldmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            sp_cost = sp_cost / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                if self.unit.skill_parameters[3] > 1.0 {
                    avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[0];
            arts_scale = 0.0;
            if self.unit.skill_damage {
                arts_scale = self.unit.skill_parameters[1];
            }
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            artsdmg = ((final_atk * atk_scale * arts_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * arts_scale * 0.05) as f64);
            dps = (hitdmg + artsdmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64) + self.unit.skill_parameters[1];
            atkbuff += self.unit.skill_parameters[0];
            // UNTRANSLATED: if self.skill_dmg: atkbuff += self.skill_params[0]
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / atk_interval * (self.unit.attack_speed + aspd) / 100.0
                * (self.unit.targets as f64);
        }
        dps
    }
}

impl std::ops::Deref for Horn {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Horn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
