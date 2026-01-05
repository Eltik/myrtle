//! DPS calculations for Walter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Walter operator implementation
pub struct Walter {
    pub unit: OperatorUnit,
}

impl Walter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Walter operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        // Apply init-time modifications from Python __init__

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// bonushits = 2 if self.module == 1 else 1
    /// maintargetscale = 1 if self.elite == 0 else self.talent1_params[0]
    /// explosionscale = 0 if self.elite == 0 else self.talent1_params[2]
    /// prob = 1 - 0.85 ** bonushits
    ///
    /// if self.skill == 0:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg_main = np.fmax(final_atk * maintargetscale - defense, final_atk * maintargetscale * 0.05)
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// explosiondmg = np.fmax(final_atk * explosionscale - defense, final_atk * explosionscale * 0.05)
    /// avghit = hitdmg_main + hitdmg + explosiondmg * prob
    /// dps = avghit / self.atk_interval * self.attack_speed/100 * self.targets
    /// if self.skill == 1:
    /// prob2 = 1 - 0.85 ** (bonushits+2)
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    ///
    /// hitdmg_main = np.fmax(final_atk * maintargetscale - defense, final_atk * maintargetscale * 0.05)
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// bonushitdmg_main = np.fmax(final_atk * maintargetscale * 0.5 - defense, final_atk * maintargetscale * 0.5 * 0.05)
    /// bonushitdmg = np.fmax(final_atk * 0.5 - defense, final_atk  * 0.5 * 0.05)
    /// skillhitdmg_main = np.fmax(final_atk * maintargetscale * skill_scale - defense, final_atk * maintargetscale * skill_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// explosiondmg = np.fmax(final_atk * explosionscale - defense, final_atk * explosionscale * 0.05)
    /// sp_cost = self.skill_cost
    /// avghit_main = (sp_cost * (hitdmg_main + bonushitdmg_main * bonushits) + hitdmg_main + (bonushits+2)*skillhitdmg_main) / (sp_cost + 1)
    /// avghit = (sp_cost * (hitdmg + bonushitdmg * bonushits) + hitdmg + (bonushits+2)*skillhitdmg) / (sp_cost + 1)
    /// avg_explosion = (sp_cost * explosiondmg * prob + explosiondmg * prob2) / (sp_cost + 1)
    /// dps = (avghit_main+avg_explosion)/self.atk_interval * self.attack_speed/100
    /// if self.targets > 1:
    /// dps += (avghit+avg_explosion)/self.atk_interval * self.attack_speed/100 * (self.targets - 1)
    ///
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval + self.skill_params[0]
    /// atkbuff = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// atk_scale = self.skill_params[2] if self.skill_dmg else 1
    /// hitdmg_main = np.fmax(final_atk * maintargetscale * atk_scale - defense, final_atk * maintargetscale * atk_scale * 0.05)
    /// #hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// bonushitdmg_main = np.fmax(final_atk * maintargetscale * atk_scale * 0.5 - defense, final_atk * maintargetscale * atk_scale * 0.05)
    /// #bonushitdmg = np.fmax(final_atk * atk_scale * 0.5 - defense, final_atk * atk_scale * 0.05)
    /// explosiondmg = np.fmax(final_atk * explosionscale - defense, final_atk * explosionscale * 0.05)
    /// dps = (hitdmg_main + bonushitdmg_main * bonushits + prob * explosiondmg)/atk_interval * self.attack_speed/100
    /// if self.skill_dmg: dps *= 4
    /// elif self.targets > 1: dps *= min(3, self.targets)
    ///
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[0]
    /// skill_scale = self.skill_params[3]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg_main = np.fmax(final_atk * maintargetscale * skill_scale - defense, final_atk * maintargetscale * skill_scale * 0.05)
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// bonushitdmg_main = np.fmax(final_atk * maintargetscale * skill_scale * 0.5 - defense, final_atk * skill_scale * maintargetscale * 0.5 * 0.05)
    /// bonushitdmg = np.fmax(final_atk * 0.5 - defense, final_atk * 0.5 * 0.05)
    /// explosiondmg = np.fmax(final_atk * explosionscale - defense, final_atk * explosionscale * 0.05)
    /// dps = (hitdmg_main + bonushitdmg_main * bonushits + explosiondmg)/5 * self.attack_speed/100
    /// if self.targets > 1:
    /// dps += (hitdmg + bonushitdmg * bonushits + explosiondmg)/self.atk_interval * self.attack_speed/100 * (self.targets-1)
    ///
    /// shadowhit = np.fmax(self.drone_atk * (1-res/100), self.drone_atk * 0.05) * self.shadows
    /// dps += shadowhit/4.25
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

        // Calculate shadows from __init__ logic
        let mut shadows: f64 = 1.0;
        shadows = 0.0;
        if self.unit.elite == 2
            && (self.unit.skill_index == 0 || self.unit.skill_index == 3)
            && self.unit.talent2_damage
        {
            shadows = 3.0;
        }
        if self.unit.elite == 2
            && (self.unit.skill_index == 0 || self.unit.skill_index == 3)
            && self.unit.talent2_damage
        {
            shadows = (((self.unit.skill_index as f64) + 1.0) as f64).min((2) as f64);
        }
        if self.unit.elite == 2
            && (self.unit.skill_index == 0 || self.unit.skill_index == 3)
            && self.unit.skill_parameters.get(1).copied().unwrap_or(0.0) == 1.0
            && self.unit.skill_index == 3
        {
            shadows -= 1.0;
        }
        if self.unit.elite == 2 && (self.unit.skill_index == 0 || self.unit.skill_index == 3) {
            shadows = if self.unit.talent2_damage { 1.0 } else { 0.0 };
        }

        let mut sp_cost: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg_main: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut bonushitdmg: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut explosiondmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut bonushitdmg_main: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut bonushits = if ((self.unit.module_index as f64) as f64) == 1.0 {
            2.0
        } else {
            1.0
        };
        let mut maintargetscale = if ((self.unit.elite as f64) as f64) == 0.0 {
            1.0
        } else {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
        };
        let mut explosionscale = if ((self.unit.elite as f64) as f64) == 0.0 {
            0.0
        } else {
            self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        };
        let mut prob = 1.0 - (0.85 as f64).powf(bonushits as f64);
        if (self.unit.skill_index as f64) == 0.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg_main = ((final_atk * maintargetscale - defense) as f64)
                .max((final_atk * maintargetscale * 0.05) as f64);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            explosiondmg = ((final_atk * explosionscale - defense) as f64)
                .max((final_atk * explosionscale * 0.05) as f64);
            avghit = hitdmg_main + hitdmg + explosiondmg * prob;
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            let mut prob2 = 1.0 - (0.85 as f64).powf((bonushits + 2.0) as f64);
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg_main = ((final_atk * maintargetscale - defense) as f64)
                .max((final_atk * maintargetscale * 0.05) as f64);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            bonushitdmg_main = ((final_atk * maintargetscale * 0.5 - defense) as f64)
                .max((final_atk * maintargetscale * 0.5 * 0.05) as f64);
            bonushitdmg = ((final_atk * 0.5 - defense) as f64).max((final_atk * 0.5 * 0.05) as f64);
            let mut skillhitdmg_main = ((final_atk * maintargetscale * skill_scale - defense)
                as f64)
                .max((final_atk * maintargetscale * skill_scale * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            explosiondmg = ((final_atk * explosionscale - defense) as f64)
                .max((final_atk * explosionscale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            let mut avghit_main = (sp_cost * (hitdmg_main + bonushitdmg_main * bonushits)
                + hitdmg_main
                + (bonushits + 2.0) * skillhitdmg_main)
                / (sp_cost + 1.0);
            avghit = (sp_cost * (hitdmg + bonushitdmg * bonushits)
                + hitdmg
                + (bonushits + 2.0) * skillhitdmg)
                / (sp_cost + 1.0);
            let mut avg_explosion =
                (sp_cost * explosiondmg * prob + explosiondmg * prob2) / (sp_cost + 1.0);
            dps = (avghit_main + avg_explosion) / (self.unit.attack_interval as f64)
                * self.unit.attack_speed
                / 100.0;
            if (self.unit.targets as f64) > 1.0 {
                dps += (avghit + avg_explosion) / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0
                    * ((self.unit.targets as f64) - 1.0);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = (self.unit.attack_interval as f64)
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            atkbuff = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            atk_scale = if self.unit.skill_damage {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            hitdmg_main = ((final_atk * maintargetscale * atk_scale - defense) as f64)
                .max((final_atk * maintargetscale * atk_scale * 0.05) as f64);
            // hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
            bonushitdmg_main = ((final_atk * maintargetscale * atk_scale * 0.5 - defense) as f64)
                .max((final_atk * maintargetscale * atk_scale * 0.05) as f64);
            // bonushitdmg = np.fmax(final_atk * atk_scale * 0.5 - defense, final_atk * atk_scale * 0.05)
            explosiondmg = ((final_atk * explosionscale - defense) as f64)
                .max((final_atk * explosionscale * 0.05) as f64);
            dps = (hitdmg_main + bonushitdmg_main * bonushits + prob * explosiondmg) / atk_interval
                * self.unit.attack_speed
                / 100.0;
            if self.unit.skill_damage {
                dps *= 4.0;
            } else if (self.unit.targets as f64) > 1.0 {
                dps *= ((3) as f64).min((self.unit.targets as f64) as f64);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            skill_scale = self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg_main = ((final_atk * maintargetscale * skill_scale - defense) as f64)
                .max((final_atk * maintargetscale * skill_scale * 0.05) as f64);
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            bonushitdmg_main = ((final_atk * maintargetscale * skill_scale * 0.5 - defense) as f64)
                .max((final_atk * skill_scale * maintargetscale * 0.5 * 0.05) as f64);
            bonushitdmg = ((final_atk * 0.5 - defense) as f64).max((final_atk * 0.5 * 0.05) as f64);
            explosiondmg = ((final_atk * explosionscale - defense) as f64)
                .max((final_atk * explosionscale * 0.05) as f64);
            dps = (hitdmg_main + bonushitdmg_main * bonushits + explosiondmg) / 5.0
                * self.unit.attack_speed
                / 100.0;
            if (self.unit.targets as f64) > 1.0 {
                dps += (hitdmg + bonushitdmg * bonushits + explosiondmg)
                    / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0
                    * ((self.unit.targets as f64) - 1.0);
            }
        }
        let mut shadowhit = ((self.unit.drone_atk * (1.0 - res / 100.0)) as f64)
            .max((self.unit.drone_atk * 0.05) as f64)
            * shadows;
        dps += shadowhit / 4.25;
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// if self.skill == 3:
    /// return(self.skill_dps(defense,res) * 6 * (5/(self.attack_speed/100)))
    /// else:
    /// return(super().total_dmg(defense,res))
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
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        if (self.unit.skill_index as f64) == 3.0 {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * 6 * (5/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(super().total_dmg(defense,res)) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for Walter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Walter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
