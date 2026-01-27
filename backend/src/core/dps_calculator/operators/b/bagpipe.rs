//! DPS calculations for Bagpipe
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Bagpipe operator implementation
pub struct Bagpipe {
    pub unit: OperatorUnit,
}

impl Bagpipe {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("module", "lowHpTarget", false, &[], &[2], 0, 0)];

    /// Creates a new Bagpipe operator
    #[allow(unused_parens)]
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
    /// atk_scale = 1.15 if self.module == 2 and self.module_dmg else 1
    /// crate = self.talent1_params[1] if self.elite > 0 else 0
    /// cdmg = self.talent1_params[0] if self.elite > 0 else 1
    /// if self.skill < 2:
    /// atkbuff = self.skill_params[0] * self.skill
    /// aspd = self.skill_params[1] * self.skill
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale * cdmg - defense, final_atk * atk_scale * cdmg * 0.05)
    /// avgdmg = crate * critdmg * min(2, self.targets) + (1-crate) * hitdmg
    /// dps = avgdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale *cdmg - defense, final_atk * atk_scale * cdmg * 0.05)
    /// skillhit = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05)
    /// skillcrit = np.fmax(final_atk * atk_scale * skill_scale *cdmg - defense, final_atk * atk_scale * skill_scale * cdmg * 0.05)
    /// avgdmg = crate * critdmg * min(2, self.targets) + (1-crate) * hitdmg
    /// avgskill = crate * skillcrit * min(2, self.targets) + (1-crate) * skillhit
    /// avgskill *= 2
    /// sp_cost = self.skill_cost / (1 + self.sp_boost) + 1.2
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = avgskill
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[1] > 1:
    /// avghit = (avgskill + (atks_per_skillactivation - 1) * avgdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (avgskill + int(atks_per_skillactivation) * avgdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[0]
    /// self.atk_interval = 1.7
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale *cdmg - defense, final_atk * atk_scale * cdmg * 0.05)
    /// avgdmg = crate * critdmg * min(2, self.targets) + (1-crate) * hitdmg
    /// dps = 3 * avgdmg/self.atk_interval * self.attack_speed/100
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut critdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut sp_cost: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        cdmg = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64);
            aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * atk_scale * cdmg - defense) as f64)
                .max((final_atk * atk_scale * cdmg * 0.05) as f64);
            avgdmg = crit_rate * critdmg * ((2) as f64).min((self.unit.targets as f64) as f64)
                + (1.0 - crit_rate) * hitdmg;
            dps = avgdmg / atk_interval * (self.unit.attack_speed + aspd) / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * atk_scale * cdmg - defense) as f64)
                .max((final_atk * atk_scale * cdmg * 0.05) as f64);
            let mut skillhit = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            let mut skillcrit = ((final_atk * atk_scale * skill_scale * cdmg - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * cdmg * 0.05) as f64);
            avgdmg = crit_rate * critdmg * ((2) as f64).min((self.unit.targets as f64) as f64)
                + (1.0 - crit_rate) * hitdmg;
            let mut avgskill =
                crit_rate * skillcrit * ((2) as f64).min((self.unit.targets as f64) as f64)
                    + (1.0 - crit_rate) * skillhit;
            avgskill *= 2.0;
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle = atk_interval / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = avgskill;
            if atks_per_skillactivation > 1.0 {
                if self.unit.skill_parameters[1] > 1.0 {
                    avghit = (avgskill + (atks_per_skillactivation - 1.0) * avgdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (avgskill + ((atks_per_skillactivation) as f64).trunc() * avgdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / atk_interval * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            atk_interval = 1.7;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * atk_scale * cdmg - defense) as f64)
                .max((final_atk * atk_scale * cdmg * 0.05) as f64);
            avgdmg = crit_rate * critdmg * ((2) as f64).min((self.unit.targets as f64) as f64)
                + (1.0 - crit_rate) * hitdmg;
            dps = 3.0 * avgdmg / atk_interval * self.unit.attack_speed / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Bagpipe {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Bagpipe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Bagpipe {
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
