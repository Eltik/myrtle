//! DPS calculations for Qiubai
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Qiubai operator implementation
pub struct Qiubai {
    pub unit: OperatorUnit,
}

impl Qiubai {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Qiubai operator
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
    ///
    /// bonus = 0.1 if self.module == 1 else 0
    /// extrascale = self.talent1_params[0] if self.elite > 0 else 0
    /// dmg = 1 + 0.1 * (self.module_lvl-1) if self.module == 1 and self.module_dmg else 1
    /// atk_scale = 1 if self.trait_dmg else 0.8
    ///
    /// if self.skill  < 2:
    /// skill_scale = self.skill_params[0]
    /// if not self.talent_dmg:
    /// extrascale = 0
    /// dmg = 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * dmg
    /// hitdmgarts = np.fmax(final_atk * extrascale * (1-res/100), final_atk * extrascale * 0.05) * dmg
    /// skilldmg = np.fmax(final_atk * (skill_scale+extrascale) * (1-res/100), final_atk * (skill_scale+extrascale) * 0.05) * dmg * self.targets
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// avghit = (hitdmg + hitdmgarts + bonusdmg) * self.skill_cost + skilldmg + bonusdmg * self.targets
    /// avghit = avghit/(self.skill_cost+1) if self.skill == 1 else hitdmg + hitdmgarts + bonusdmg
    /// dps = avghit/self.atk_interval * (self.attack_speed)/100
    /// ####the actual skills
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[0]
    /// aspd = self.skill_params[1] * self.skill_params[2] if self.skill_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// try: extrascale *= self.skill_params[3]
    /// except: pass
    /// atk_cycle = self.atk_interval / (self.attack_speed + aspd) * 100
    /// bind_chance = self.talent2_params[0]
    /// counting_hits = int(1.5/atk_cycle) + 1
    /// chance_to_attack_bind = 1 - (1-bind_chance) ** counting_hits
    /// if not self.talent_dmg and not self.talent2_dmg: #talent not active
    /// extrascale = 0
    /// dmg = 1
    /// elif self.module_dmg and not self.talent_dmg: #vs slow + self applied
    /// dmg = (dmg - 1) * chance_to_attack_bind + 1
    /// elif not self.module_dmg and self.talent_dmg: #vs slow OR bind
    /// dmg = 1
    /// elif not self.module_dmg and not self.talent_dmg: #only self applied
    /// extrascale *= chance_to_attack_bind
    /// dmg = 1
    /// hitdmgarts = np.fmax(final_atk * (1+extrascale) * (1-res/100), final_atk * (1+extrascale) * 0.05) * dmg
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// dps = (hitdmgarts+bonusdmg)/self.atk_interval * (self.attack_speed+aspd)/100 * min(3, self.targets)
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

        let mut dps: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut bonusdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut extrascale: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dmg: f64 = 0.0;

        let mut bonus = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        extrascale = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        dmg = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.0 + 0.1 * (((self.unit.module_level as f64) as f64) - 1.0)
        } else {
            1.0
        };
        atk_scale = if self.unit.trait_damage { 1.0 } else { 0.8 };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            if !self.unit.talent_damage {
                extrascale = 0.0;
                dmg = 1.0;
            }
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg;
            hitdmgarts = ((final_atk * extrascale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * extrascale * 0.05) as f64)
                * dmg;
            skilldmg = ((final_atk * (skill_scale + extrascale) * (1.0 - res / 100.0)) as f64)
                .max((final_atk * (skill_scale + extrascale) * 0.05) as f64)
                * dmg
                * (self.unit.targets as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            avghit = (hitdmg + hitdmgarts + bonusdmg) * (self.unit.skill_cost as f64)
                + skilldmg
                + bonusdmg * (self.unit.targets as f64);
            avghit = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                avghit / ((self.unit.skill_cost as f64) + 1.0)
            } else {
                hitdmg + hitdmgarts + bonusdmg
            };
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
            // ###the actual skills
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            aspd = if self.unit.skill_damage {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                    * self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            // Python: try: extrascale *= self.skill_params[3] except: pass
            if let Some(val) = self.unit.skill_parameters.get(3) {
                extrascale *= val;
            }
            let mut atk_cycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed + aspd) * 100.0;
            let mut bind_chance = self.unit.talent2_parameters.first().copied().unwrap_or(0.0);
            let mut counting_hits = ((1.5 / atk_cycle) as f64).trunc() + 1.0;
            let mut chance_to_attack_bind =
                1.0 - ((1.0 - bind_chance) as f64).powf(counting_hits as f64);
            if !self.unit.talent_damage && !self.unit.talent2_damage {
                // talent not active
                extrascale = 0.0;
                dmg = 1.0;
            } else if self.unit.module_damage && !self.unit.talent_damage {
                dmg = (dmg - 1.0) * chance_to_attack_bind + 1.0;
            } else if !self.unit.module_damage && self.unit.talent_damage {
                dmg = 1.0;
            } else if !self.unit.module_damage && !self.unit.talent_damage {
                extrascale *= chance_to_attack_bind;
                dmg = 1.0;
            }
            hitdmgarts = ((final_atk * (1.0 + extrascale) * (1.0 - res / 100.0)) as f64)
                .max((final_atk * (1.0 + extrascale) * 0.05) as f64)
                * dmg;
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            dps = (hitdmgarts + bonusdmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0
                * ((3) as f64).min((self.unit.targets as f64) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Qiubai {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Qiubai {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
