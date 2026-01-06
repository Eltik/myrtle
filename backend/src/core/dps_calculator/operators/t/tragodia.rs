//! DPS calculations for Tragodia
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Tragodia operator implementation
pub struct Tragodia {
    pub unit: OperatorUnit,
}

impl Tragodia {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Tragodia operator
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
    /// nerv_factor = self.talent1_params[0]
    /// nerv_aoe = self.talent1_params[1]
    /// mod_factor = 1.18 if self.module == 1 and (not self.trait_dmg or self.module_dmg) else 1
    /// ele_gauge = 1000 if self.trait_dmg else 2000
    /// atkbuff = self.skill_params[0] if self.skill == 3 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    ///
    /// if self.skill == 0:
    /// ele_dps = 6000/(10 + (ele_gauge / (final_atk * nerv_factor * mod_factor / self.atk_interval * self.attack_speed / 100)))
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100 + ele_dps
    ///
    /// if self.skill == 1:
    /// skilldmg = np.fmax(final_atk * self.skill_params[0] * (1-res/100), final_atk * self.skill_params[0] * 0.05) * 2
    /// nerv_dps = (final_atk * nerv_factor * mod_factor * self.skill_cost + 2 * final_atk * nerv_factor * mod_factor * self.skill_params[1])/(self.skill_cost+1)/ self.atk_interval * self.attack_speed  / 100
    /// ele_dps = 6000/(10+ele_gauge/nerv_dps)
    /// dps = (skilldmg + hitdmg * self.skill_cost)/(self.skill_cost+1)/ self.atk_interval * self.attack_speed / 100 + ele_dps
    ///
    /// if self.skill == 2:
    /// skill_factor = self.skill_params[0]
    /// artsdmg = np.fmax(final_atk * skill_factor * (1-res/100), final_atk * skill_factor * 0.05)
    /// dps = 12 * artsdmg / 25
    ///
    /// if self.skill_dmg:
    /// ele_dps = 6000/(10 + (ele_gauge / (final_atk * nerv_factor * mod_factor / self.atk_interval * (self.attack_speed+self.skill_params[7]) / 100)))
    /// dps += hitdmg / self.atk_interval * (self.attack_speed+self.skill_params[7]) / 100 + ele_dps
    /// else:
    /// if 12 * 0.25 * final_atk * mod_factor < ele_gauge:
    /// dps += 3000 / 25
    /// else:
    /// dps += 6000 / 25
    ///
    /// if self.skill == 3:
    /// ele_dps = 6000/(6.666 + (ele_gauge / (final_atk * nerv_factor * mod_factor / self.atk_interval * self.attack_speed / 100 + final_atk * 0.1 * mod_factor)))
    /// dps = hitdmg / self.atk_interval * self.attack_speed / 100 + ele_dps
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
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut ele_dps: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        let mut nerv_factor = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        let mut nerv_aoe = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        let mut mod_factor = if ((self.unit.module_index as f64) as f64) == 1.0
            && (!self.unit.trait_damage || self.unit.module_damage)
        {
            1.18
        } else {
            1.0
        };
        let mut ele_gauge = if self.unit.trait_damage {
            1000.0
        } else {
            2000.0
        };
        atkbuff = if ((self.unit.skill_index as f64) as f64) == 3.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
        if (self.unit.skill_index as f64) == 0.0 {
            ele_dps = 6000.0
                / (10.0
                    + (ele_gauge
                        / (final_atk * nerv_factor * mod_factor
                            / (self.unit.attack_interval as f64)
                            * self.unit.attack_speed
                            / 100.0)));
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                + ele_dps;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skilldmg = ((final_atk
                * self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * (1.0 - res / 100.0)) as f64)
                .max(
                    (final_atk * self.unit.skill_parameters.first().copied().unwrap_or(0.0) * 0.05)
                        as f64,
                )
                * 2.0;
            let mut nerv_dps =
                (final_atk * nerv_factor * mod_factor * (self.unit.skill_cost as f64)
                    + 2.0
                        * final_atk
                        * nerv_factor
                        * mod_factor
                        * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
                    / ((self.unit.skill_cost as f64) + 1.0)
                    / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0;
            ele_dps = 6000.0 / (10.0 + ele_gauge / nerv_dps);
            dps = (skilldmg + hitdmg * (self.unit.skill_cost as f64))
                / ((self.unit.skill_cost as f64) + 1.0)
                / (self.unit.attack_interval as f64)
                * self.unit.attack_speed
                / 100.0
                + ele_dps;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            let mut skill_factor = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut artsdmg = ((final_atk * skill_factor * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_factor * 0.05) as f64);
            dps = 12.0 * artsdmg / 25.0;
            if self.unit.skill_damage {
                ele_dps = 6000.0
                    / (10.0
                        + (ele_gauge
                            / (final_atk * nerv_factor * mod_factor
                                / (self.unit.attack_interval as f64)
                                * (self.unit.attack_speed
                                    + self.unit.skill_parameters.get(7).copied().unwrap_or(0.0))
                                / 100.0)));
                dps += hitdmg / (self.unit.attack_interval as f64)
                    * (self.unit.attack_speed
                        + self.unit.skill_parameters.get(7).copied().unwrap_or(0.0))
                    / 100.0
                    + ele_dps;
            } else {
                if 12.0 * 0.25 * final_atk * mod_factor < ele_gauge {
                    dps += 3000.0 / 25.0;
                } else {
                    dps += 6000.0 / 25.0;
                }
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            ele_dps = 6000.0
                / (6.666
                    + (ele_gauge
                        / (final_atk * nerv_factor * mod_factor
                            / (self.unit.attack_interval as f64)
                            * self.unit.attack_speed
                            / 100.0
                            + final_atk * 0.1 * mod_factor)));
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                + ele_dps;
        }
        return dps;
    }
}

impl std::ops::Deref for Tragodia {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Tragodia {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Tragodia {
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
