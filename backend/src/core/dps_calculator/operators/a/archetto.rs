//! DPS calculations for Archetto
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Archetto operator implementation
pub struct Archetto {
    pub unit: OperatorUnit,
}

impl Archetto {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Archetto operator
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
    /// aspd = 8 if self.module == 2 and self.module_dmg else 0
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// recovery_interval = max(self.talent1_params) if self.elite > 0 else 10000000
    /// if self.module == 1 and self.talent_dmg and self.module_lvl > 1:
    /// recovery_interval -= 0.3 if self.module_lvl == 2 else 0.4
    ///
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// skill_scale2= self.skill_params[1]
    /// sp_cost = self.skill_cost
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// aoedmg = np.fmax(final_atk * skill_scale2 * atk_scale - defense, final_atk * skill_scale2 * atk_scale * 0.05)
    ///
    /// #figuring out the chance of the talent to activate during downtime
    /// base_cycle_time = (sp_cost+1)/((self.attack_speed+aspd)/100)
    /// talents_per_base_cycle = base_cycle_time / recovery_interval
    /// failure_rate = 1.8 / (sp_cost + 1)  #1 over sp cost because thats the time the skill would technically be ready, the bonus is for sp lockout. (basis is a video where each attack had 14 frames, but it was 25 frames blocked)
    /// talents_per_base_cycle *= 1-failure_rate
    /// new_spcost = np.fmax(1,sp_cost - talents_per_base_cycle)
    /// hitdps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) * (new_spcost-1)/new_spcost
    /// skilldps = skilldmg/(self.atk_interval/((self.attack_speed+aspd)/100)) /new_spcost
    /// aoedps = aoedmg/(self.atk_interval/((self.attack_speed+aspd)/100)) /new_spcost *(min(self.targets,4)-1)
    /// dps = hitdps + skilldps + aoedps
    ///
    /// if self.skill == 2:
    /// sprecovery = 1/recovery_interval + (self.attack_speed+aspd)/100
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// targets = min(5, self.targets)
    /// totalhits = [5,9,12,14,15]
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) + sprecovery/self.skill_cost * skilldmg * totalhits[targets-1]
    ///
    /// if self.skill in [0,3]:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]*self.skill/3) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05) * (1 + self.skill * 2 / 3)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    /// if self.skill == 3: dps *= min(self.targets, 2)
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

        let mut sp_cost: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut skilldmg: f64 = 0.0;

        aspd = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            8.0
        } else {
            0.0
        };
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        let mut recovery_interval = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit
                .talent1_parameters
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max)
        } else {
            10000000.0
        };
        if (self.unit.module_index as f64) == 1.0
            && self.unit.talent_damage
            && (self.unit.module_level as f64) > 1.0
        {
            recovery_interval -= if ((self.unit.module_level as f64) as f64) == 2.0 {
                0.3
            } else {
                0.4
            };
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut skill_scale2 = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            sp_cost = (self.unit.skill_cost as f64);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            let mut aoedmg = ((final_atk * skill_scale2 * atk_scale - defense) as f64)
                .max((final_atk * skill_scale2 * atk_scale * 0.05) as f64);
            // figuring out the chance of the talent to activate during downtime
            let mut base_cycle_time = (sp_cost + 1.0) / ((self.unit.attack_speed + aspd) / 100.0);
            let mut talents_per_base_cycle = base_cycle_time / recovery_interval;
            let mut failure_rate = 1.8 / (sp_cost + 1.0);
            talents_per_base_cycle *= 1.0 - failure_rate;
            let mut new_spcost = ((1) as f64).max((sp_cost - talents_per_base_cycle) as f64);
            let mut hitdps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * (new_spcost - 1.0)
                / new_spcost;
            let mut skilldps = skilldmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                / new_spcost;
            let mut aoedps = aoedmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                / new_spcost
                * (((self.unit.targets as f64) as f64).min((4) as f64) - 1.0);
            dps = hitdps + skilldps + aoedps;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut sprecovery = 1.0 / recovery_interval + (self.unit.attack_speed + aspd) / 100.0;
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            let mut targets = ((5) as f64).min((self.unit.targets as f64) as f64);
            let mut totalhits = [5.0, 9.0, 12.0, 14.0, 15.0];
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                + sprecovery / (self.unit.skill_cost as f64)
                    * skilldmg
                    * totalhits[((targets) as usize).saturating_sub(1)];
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 3.0)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * (1.0 + (self.unit.skill_index as f64) * 2.0 / 3.0);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
            if (self.unit.skill_index as f64) == 3.0 {
                dps *= ((self.unit.targets as f64) as f64).min((2) as f64);
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Archetto {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Archetto {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Archetto {
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
