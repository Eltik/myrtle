//! DPS calculations for Vina
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Vina operator implementation
pub struct Vina {
    pub unit: OperatorUnit,
}

impl Vina {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Vina operator
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
    /// value = 0.1 if self.module == 2 and self.module_dmg else 0
    /// fragile = max(value, self.buff_fragile)
    /// dmg_scale = 1 + 0.05 * self.module_lvl if self.module == 2 and self.module_lvl > 1 and self.talent2_dmg else 1
    /// atkbuff = self.talent1_params[1] * self.count
    /// aspd = 8 if self.module == 1 and self.module_dmg else 0
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// hits = self.skill_cost
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-res/100), final_atk * 0.05)
    /// skilldmgarts = np.fmax(final_atk * skill_scale *(1-res/100), final_atk * skill_scale * 1)
    /// if self.skill == 0: skilldmgarts = hitdmgarts
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/((self.attack_speed)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmgarts
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmgarts + int(atks_per_skillactivation) * hitdmgarts) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/(self.atk_interval/((self.attack_speed+aspd)/100))
    ///
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) * min(self.targets,2)
    /// if self.skill == 3:
    /// atk_interval = self.atk_interval + self.skill_params[0]
    /// atkbuff += self.skill_params[1]
    /// maxtargets = self.skill_params[2]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-res/100), final_atk * 1)
    /// hitdmg_lion = np.fmax(self.drone_atk *(1-res/100), self.drone_atk * 1)
    /// dps = hitdmgarts/(atk_interval/((self.attack_speed+aspd)/100)) * min(self.targets,maxtargets) + hitdmg_lion/self.drone_atk_interval * min(self.targets, self.count)
    /// return dps * dmg_scale * (1+fragile)/(1+self.buff_fragile)
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

        let mut aspd: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;

        let mut value =
            if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
                0.1
            } else {
                0.0
            };
        let mut fragile = ((value) as f64).max((self.unit.buff_fragile) as f64);
        let mut dmg_scale = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && self.unit.talent2_damage
        {
            1.0 + 0.05 * ((self.unit.module_level as f64) as f64)
        } else {
            1.0
        };
        atkbuff = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0) * 1.0 /* self.count - needs manual implementation */;
        aspd = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            8.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut hits = (self.unit.skill_cost as f64);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut skilldmgarts = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 1.0) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmgarts = hitdmgarts;
            }
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmgarts;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmgarts + ((atks_per_skillactivation) as f64).trunc() * hitdmgarts)
                    / (((atks_per_skillactivation) as f64).trunc() + 1.0);
            }
            dps = avghit
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64)
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            atkbuff += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            let mut maxtargets = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 1.0) as f64);
            let mut hitdmg_lion = ((self.unit.drone_atk * (1.0 - res / 100.0)) as f64)
                .max((self.unit.drone_atk * 1.0) as f64);
            dps = hitdmgarts / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0))
                * ((self.unit.targets as f64) as f64).min((maxtargets) as f64)
                + hitdmg_lion / (self.unit.drone_atk_interval as f64)
                    * ((self.unit.targets as f64) as f64)
                        .min((1.0/* self.count - needs manual implementation */) as f64);
        }
        return dps * dmg_scale * (1.0 + fragile) / (1.0 + self.unit.buff_fragile);
    }
}

impl std::ops::Deref for Vina {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Vina {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
