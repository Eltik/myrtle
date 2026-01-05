//! DPS calculations for Diamante
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Diamante operator implementation
pub struct Diamante {
    pub unit: OperatorUnit,
}

impl Diamante {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Diamante operator
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
    /// burst_scale = 1.1 if self.module == 1 else 1
    /// if self.skill in [0,2]:
    /// atkbuff = self.talent1_params[0] if self.talent_dmg and self.skill_dmg and self.skill == 2 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// skill_scale = self.skill_params[1] if self.talent_dmg and self.skill_dmg and self.skill == 2 else 0
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// eledmg =  np.fmax(final_atk * 0 * (1-res/100), final_atk * skill_scale) /(1+self.buff_fragile) * burst_scale
    /// dps = (hitdmg+eledmg) / self.atk_interval * (self.attack_speed + self.skill_params[0]) / 100 * min(self.targets,2)
    ///
    /// if self.skill == 1:
    /// atkbuff = self.skill_params[0]
    /// ele_application = self.skill_params[1]
    /// skill_scale = self.skill_params[2]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// atkbuff += self.talent1_params[0]
    /// final_atk_necro = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// elemental_health = 2000 if not self.talent_dmg and not self.skill_dmg else 1000
    /// time_to_apply_necrosis = elemental_health / (final_atk * ele_application / self.atk_interval * (self.attack_speed) / 100)
    /// fallout_dps = 12000 / (time_to_apply_necrosis + 15) /(1+self.buff_fragile)
    ///
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmg_necro = np.fmax(final_atk_necro * (1-res/100), final_atk_necro * 0.05)
    /// eledmg_necro =  np.fmax(final_atk_necro * 0 * (1-res/100), final_atk_necro * skill_scale) /(1+self.buff_fragile)
    /// avg_hitdmg = hitdmg * time_to_apply_necrosis / (time_to_apply_necrosis + 15) + hitdmg_necro * 15 / (time_to_apply_necrosis + 15)
    /// avg_eledmg = eledmg_necro * 15 / (time_to_apply_necrosis + 15)
    ///
    /// if not self.trait_dmg: dps = (hitdmg) / self.atk_interval * (self.attack_speed) / 100
    /// elif not self.talent_dmg and not self.skill_dmg: dps = fallout_dps + (avg_hitdmg + avg_eledmg * burst_scale) / self.atk_interval * (self.attack_speed) / 100
    /// elif self.talent_dmg ^ self.skill_dmg: dps = fallout_dps + (avg_hitdmg + avg_eledmg * burst_scale) / self.atk_interval * (self.attack_speed) / 100
    /// else: dps = (hitdmg_necro + eledmg_necro) * burst_scale / self.atk_interval * (self.attack_speed) / 100
    ///
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

        let mut hitdmg: f64 = 0.0;
        let mut burst_scale: f64 = 0.0;
        let mut eledmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        burst_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
            1.1
        } else {
            1.0
        };
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff = if self.unit.talent_damage
                && self.unit.skill_damage
                && ((self.unit.skill_index as f64) as f64) == 2.0
            {
                self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            skill_scale = if self.unit.talent_damage
                && self.unit.skill_damage
                && ((self.unit.skill_index as f64) as f64) == 2.0
            {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            eledmg = ((final_atk * 0.0 * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale) as f64)
                / (1.0 + self.unit.buff_fragile)
                * burst_scale;
            dps = (hitdmg + eledmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut ele_application = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            atkbuff += self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
            let mut final_atk_necro =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut elemental_health = if !self.unit.talent_damage && !self.unit.skill_damage {
                2000.0
            } else {
                1000.0
            };
            let mut time_to_apply_necrosis = elemental_health
                / (final_atk * ele_application / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed)
                    / 100.0);
            let mut fallout_dps =
                12000.0 / (time_to_apply_necrosis + 15.0) / (1.0 + self.unit.buff_fragile);
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut hitdmg_necro = ((final_atk_necro * (1.0 - res / 100.0)) as f64)
                .max((final_atk_necro * 0.05) as f64);
            let mut eledmg_necro = ((final_atk_necro * 0.0 * (1.0 - res / 100.0)) as f64)
                .max((final_atk_necro * skill_scale) as f64)
                / (1.0 + self.unit.buff_fragile);
            let mut avg_hitdmg = hitdmg * time_to_apply_necrosis / (time_to_apply_necrosis + 15.0)
                + hitdmg_necro * 15.0 / (time_to_apply_necrosis + 15.0);
            let mut avg_eledmg = eledmg_necro * 15.0 / (time_to_apply_necrosis + 15.0);
            if !self.unit.trait_damage {
                dps = (hitdmg) / (self.unit.attack_interval as f64) * (self.unit.attack_speed)
                    / 100.0;
            } else if !self.unit.talent_damage && !self.unit.skill_damage {
                dps = fallout_dps
                    + (avg_hitdmg + avg_eledmg * burst_scale) / (self.unit.attack_interval as f64)
                        * (self.unit.attack_speed)
                        / 100.0;
            } else if self.unit.talent_damage ^ self.unit.skill_damage {
                dps = fallout_dps
                    + (avg_hitdmg + avg_eledmg * burst_scale) / (self.unit.attack_interval as f64)
                        * (self.unit.attack_speed)
                        / 100.0;
            } else {
                dps = (hitdmg_necro + eledmg_necro) * burst_scale
                    / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed)
                    / 100.0;
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Diamante {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Diamante {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Diamante {
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
