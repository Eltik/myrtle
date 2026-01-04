//! DPS calculations for Warmy
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Warmy operator implementation
pub struct Warmy {
    pub unit: OperatorUnit,
}

impl Warmy {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Warmy operator
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
    /// falloutdmg = 7000
    /// burst_scale = 1.1 if ((self.skill == 2 and self.skill_dmg) or (self.skill == 1 and self.trait_dmg)) and self.module == 1 else 1
    /// if self.skill == 0:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 1:
    /// aspd = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.elite > 0: falloutdmg += self.talent1_params[0] * final_atk
    /// newres = np.fmax(0,res-20)
    /// elegauge = 1000 if self.skill_dmg else 2000
    /// hitdmg1 = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk * (1-newres/100), final_atk * 0.05)
    /// dpsNorm = hitdmg1/self.atk_interval * (self.attack_speed+aspd)/100
    /// dpsFallout = hitdmg2/self.atk_interval * (self.attack_speed+aspd)/100
    /// timeToFallout = elegauge/(dpsNorm * 0.15)
    /// dps = (dpsNorm * timeToFallout + dpsFallout * burst_scale * 10 + falloutdmg)/(timeToFallout + 10)
    /// if not self.trait_dmg: dps = dpsNorm
    ///
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmgele = final_atk * 0.5
    /// hitdmg = hitdmgarts + hitdmgele if self.skill_dmg else hitdmgarts
    /// dps = hitdmg* burst_scale/2.5 * self.attack_speed/100 * min(self.targets,3)
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

        let mut atkbuff: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut burst_scale: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        let mut falloutdmg = 7000.0;
        burst_scale = if ((((self.unit.skill_index as f64) as f64) == 2.0
            && self.unit.skill_damage)
            || (((self.unit.skill_index as f64) as f64) == 1.0 && self.unit.trait_damage))
            && ((self.unit.module_index as f64) as f64) == 1.0
        {
            1.1
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            aspd = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            if (self.unit.elite as f64) > 0.0 {
                falloutdmg +=
                    self.unit.talent1_parameters.first().copied().unwrap_or(0.0) * final_atk;
            }
            newres = ((0) as f64).max((res - 20.0) as f64);
            let mut elegauge = if self.unit.skill_damage {
                1000.0
            } else {
                2000.0
            };
            let mut hitdmg1 =
                ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut hitdmg2 =
                ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut dps_norm = hitdmg1 / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
            let mut dps_fallout = hitdmg2 / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
            let mut time_to_fallout = elegauge / (dps_norm * 0.15);
            dps = (dps_norm * time_to_fallout + dps_fallout * burst_scale * 10.0 + falloutdmg)
                / (time_to_fallout + 10.0);
            if !self.unit.trait_damage {
                dps = dps_norm;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut hitdmgele = final_atk * 0.5;
            hitdmg = if self.unit.skill_damage {
                hitdmgarts + hitdmgele
            } else {
                hitdmgarts
            };
            dps = hitdmg * burst_scale / 2.5 * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((3) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Warmy {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Warmy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
