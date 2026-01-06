//! DPS calculations for Mountain
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Mountain operator implementation
pub struct Mountain {
    pub unit: OperatorUnit,
}

impl Mountain {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Mountain operator
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
    /// crate = self.talent1_params[1]
    /// cdmg = self.talent1_params[0]
    /// aspd = 10 if self.module == 2 and self.module_dmg else 0
    ///
    /// if self.skill == 1:
    /// atk_scale = self.skill_params[0]
    /// hits = self.skill_cost
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// normalhitdmg = np.fmax(final_atk - defense, final_atk*0.05)
    /// crithitdmg = np.fmax(final_atk * cdmg-defense, final_atk*cdmg*0.05)
    /// avghit = crate * crithitdmg + (1-crate) * normalhitdmg
    /// normalskilldmg = np.fmax(final_atk * atk_scale -defense, final_atk*0.05)
    /// critskilldmg = np.fmax(final_atk * atk_scale * cdmg - defense, final_atk * cdmg * atk_scale * 0.05)
    /// avgskill = crate * critskilldmg + (1-crate) * normalskilldmg
    /// avgskill = avgskill * min(self.targets,2)
    /// avgdmg = (hits * avghit + avgskill) / (hits + 1)
    /// dps = avgdmg/(self.atk_interval/((self.attack_speed + aspd)/100))
    /// if self.skill in [0,2]:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]*self.skill/2) + self.buff_atk_flat
    /// normalhitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// crithitdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// avgdmg = normalhitdmg * (1-crate) + crithitdmg * crate
    /// dps = avgdmg/(self.atk_interval/((self.attack_speed + aspd)/100)) * min(self.targets , (1+self.skill/2))
    /// if self.skill == 3:
    /// atk_interval = self.atk_interval * 1.7
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[1]) + self.buff_atk_flat
    /// normalhitdmg = np.fmax(final_atk-defense, final_atk*0.05)
    /// crithitdmg = np.fmax(final_atk*cdmg-defense, final_atk*cdmg*0.05)
    /// crate = self.skill_params[2]
    /// targets = self.skill_params[4]
    /// avgdmg = normalhitdmg * (1-crate) + crithitdmg * crate
    /// dps = 2 * avgdmg/(atk_interval/((self.attack_speed + aspd)/100)) * min(self.targets,targets)
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

        let mut normalhitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut crithitdmg: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut avghit: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;

        let mut crit_rate = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        cdmg = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        aspd = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            10.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            atk_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut hits = (self.unit.skill_cost as f64);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            normalhitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            crithitdmg =
                ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
            avghit = crit_rate * crithitdmg + (1.0 - crit_rate) * normalhitdmg;
            let mut normalskilldmg =
                ((final_atk * atk_scale - defense) as f64).max((final_atk * 0.05) as f64);
            let mut critskilldmg = ((final_atk * atk_scale * cdmg - defense) as f64)
                .max((final_atk * cdmg * atk_scale * 0.05) as f64);
            let mut avgskill = crit_rate * critskilldmg + (1.0 - crit_rate) * normalskilldmg;
            avgskill = avgskill * ((self.unit.targets as f64) as f64).min((2) as f64);
            avgdmg = (hits * avghit + avgskill) / (hits + 1.0);
            dps = avgdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 2.0)
                + self.unit.buff_atk_flat;
            normalhitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            crithitdmg =
                ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
            avgdmg = normalhitdmg * (1.0 - crit_rate) + crithitdmg * crit_rate;
            dps = avgdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * ((self.unit.targets as f64) as f64)
                    .min((1.0 + (self.unit.skill_index as f64) / 2.0) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64) * 1.7;
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            normalhitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            crithitdmg =
                ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
            crit_rate = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            let mut targets = self.unit.skill_parameters.get(4).copied().unwrap_or(0.0);
            avgdmg = normalhitdmg * (1.0 - crit_rate) + crithitdmg * crit_rate;
            dps = 2.0 * avgdmg / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0))
                * ((self.unit.targets as f64) as f64).min((targets) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Mountain {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Mountain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Mountain {
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
