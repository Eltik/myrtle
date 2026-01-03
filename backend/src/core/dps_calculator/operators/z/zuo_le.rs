//! DPS calculations for ZuoLe
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// ZuoLe operator implementation
pub struct ZuoLe {
    pub unit: OperatorUnit,
}

impl ZuoLe {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new ZuoLe operator
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
    /// sp_recovery = 1
    /// aspd = max(self.talent1_params) if self.talent_dmg and self.talent2_dmg else 0
    /// if self.talent_dmg and self.talent2_dmg: sp_recovery += self.talent1_params[2]
    /// if self.elite == 2:
    /// sp_recovery += self.talent2_params[2] / self.atk_interval * (self.attack_speed+aspd)/100 if self.talent_dmg and self.talent2_dmg else self.talent2_params[0] / self.atk_interval * (self.attack_speed+aspd)/100
    /// tal_scale = 0.9 + 0.1 * self.module_lvl if self.module == 2 and self.talent2_dmg and self.talent_dmg else 1
    /// apply_rate = self.talent2_params[2] if self.talent_dmg and self.talent2_dmg and self.elite == 2 else 0.2
    ///
    /// if self.skill == 0:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk * tal_scale - defense, final_atk * tal_scale * 0.05)
    /// hitdmg = hitdmg * (1-apply_rate) + hitdmg2 * apply_rate
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 1:
    /// atk_scale = self.skill_params[0]
    /// hits = 3 if self.talent_dmg and self.talent2_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)
    /// hitdmg2 = np.fmax(final_atk * tal_scale - defense, final_atk * tal_scale * 0.05)
    /// hitdmg = hitdmg * (1-apply_rate) + hitdmg2 * apply_rate
    /// sp_cost = self.skill_cost / (sp_recovery + self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/((self.attack_speed + aspd)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg * hits
    /// if atks_per_skillactivation > 1:
    /// if atk_scale > 1.41:
    /// avghit = (skilldmg * hits  + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg * hits  + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk * tal_scale - defense, final_atk * tal_scale * 0.05)
    /// hitdmg = hitdmg * (1-apply_rate) + hitdmg2 * apply_rate
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100 * min(self.targets, 2)
    /// if self.skill == 3:
    /// atk_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk * tal_scale - defense, final_atk * tal_scale * 0.05)
    /// hitdmg = hitdmg * (1-apply_rate) + hitdmg2 * apply_rate
    /// skilldmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)
    /// skilldmg2= np.fmax(2*final_atk * atk_scale - defense, 2*final_atk* atk_scale * 0.05)
    /// sp_cost = self.skill_cost / (sp_recovery + self.sp_boost) + 1.2 #sp lockout
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// dps += (6 * skilldmg + skilldmg2) / sp_cost * min(self.targets,3)
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
        let defense = enemy.defense;
        let res = enemy.res;

        let mut skilldmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg2: f64 = 0.0;

        let mut sp_recovery = 1.0;
        aspd = if self.unit.talent_damage && self.unit.talent2_damage {
            self.unit
                .talent1_parameters
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max)
        } else {
            0.0
        };
        // UNTRANSLATED: if self.talent_dmg and self.talent2_dmg: sp_recovery += self.talent1_params[2]
        if (self.unit.elite as f64) == 2.0 {
            sp_recovery += if self.unit.talent_damage && self.unit.talent2_damage {
                self.unit.talent2_parameters[2] / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed + aspd)
                    / 100.0
            } else {
                self.unit.talent2_parameters[0] / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed + aspd)
                    / 100.0
            };
        }
        let mut tal_scale = if ((self.unit.module_index as f64) as f64) == 2.0
            && self.unit.talent2_damage
            && self.unit.talent_damage
        {
            0.9 + 0.1 * ((self.unit.module_level as f64) as f64)
        } else {
            1.0
        };
        let mut apply_rate = if self.unit.talent_damage
            && self.unit.talent2_damage
            && ((self.unit.elite as f64) as f64) == 2.0
        {
            self.unit.talent2_parameters[2]
        } else {
            0.2
        };
        if (self.unit.skill_index as f64) == 0.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmg2 = ((final_atk * tal_scale - defense) as f64)
                .max((final_atk * tal_scale * 0.05) as f64);
            hitdmg = hitdmg * (1.0 - apply_rate) + hitdmg2 * apply_rate;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            atk_scale = self.unit.skill_parameters[0];
            let mut hits = if self.unit.talent_damage && self.unit.talent2_damage {
                3.0
            } else {
                1.0
            };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            hitdmg2 = ((final_atk * tal_scale - defense) as f64)
                .max((final_atk * tal_scale * 0.05) as f64);
            hitdmg = hitdmg * (1.0 - apply_rate) + hitdmg2 * apply_rate;
            sp_cost =
                (self.unit.skill_cost as f64) / (sp_recovery + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg * hits;
            if atks_per_skillactivation > 1.0 {
                if atk_scale > 1.41 {
                    avghit = (skilldmg * hits + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg * hits
                        + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters[0];
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmg2 = ((final_atk * tal_scale - defense) as f64)
                .max((final_atk * tal_scale * 0.05) as f64);
            hitdmg = hitdmg * (1.0 - apply_rate) + hitdmg2 * apply_rate;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_scale = self.unit.skill_parameters[0];
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmg2 = ((final_atk * tal_scale - defense) as f64)
                .max((final_atk * tal_scale * 0.05) as f64);
            hitdmg = hitdmg * (1.0 - apply_rate) + hitdmg2 * apply_rate;
            skilldmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut skilldmg2 = ((2.0 * final_atk * atk_scale - defense) as f64)
                .max((2.0 * final_atk * atk_scale * 0.05) as f64);
            sp_cost =
                (self.unit.skill_cost as f64) / (sp_recovery + (self.unit.sp_boost as f64)) + 1.2;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
            dps += (6.0 * skilldmg + skilldmg2) / sp_cost
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for ZuoLe {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ZuoLe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
