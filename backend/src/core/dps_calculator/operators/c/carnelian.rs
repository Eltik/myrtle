//! DPS calculations for Carnelian
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Carnelian operator implementation
pub struct Carnelian {
    pub unit: OperatorUnit,
}

impl Carnelian {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Carnelian operator
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
    /// atk_scale = 1.15 if self.module == 2 and self.module_dmg else 1
    /// if self.skill == 0: return (defense * 0)
    /// if self.skill == 1:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * self.targets
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval + self.skill_params[0]
    /// atkbuff = self.skill_params[2] if self.skill_dmg else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmg/atk_interval * self.attack_speed/100 * self.targets
    /// if self.skill == 3:
    /// maxatkbuff = self.skill_params[0]
    /// duration = 21
    /// totalatks = 1 + int(duration / (self.atk_interval/(self.attack_speed/100))) # +1 because the first attack is already at 0
    /// totalduration = totalatks * (self.atk_interval/(self.attack_speed/100))
    /// damage = 0
    /// bonusscaling = 5 if self.skill_dmg else 0
    /// for i in range(totalatks):
    /// final_atk = self.atk * (1 + self.buff_atk + i * (self.atk_interval/(self.attack_speed/100)) /21 * maxatkbuff) + self.buff_atk_flat
    /// damage += np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05) * (1+ min(bonusscaling,i) * 0.2)
    /// dps = damage/totalduration * self.targets
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

        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            return (defense * 0.0);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = (self.unit.attack_interval as f64)
                + self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            atkbuff = if self.unit.skill_damage {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps =
                hitdmg / atk_interval * self.unit.attack_speed / 100.0 * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut maxatkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut duration = 21.0;
            let mut totalatks = 1.0
                + ((duration
                    / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0)))
                    as f64)
                    .trunc();
            let mut totalduration =
                totalatks * ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
            let mut damage = 0.0;
            let mut bonusscaling = if self.unit.skill_damage { 5.0 } else { 0.0 };
            // Implement for loop: for i in range(totalatks):
            for _i in 0..(totalatks as i32) {
                let i = _i as f64;
                final_atk = self.unit.atk
                    * (1.0
                        + self.unit.buff_atk
                        + i * ((self.unit.attack_interval as f64)
                            / (self.unit.attack_speed / 100.0))
                            / 21.0
                            * maxatkbuff)
                    + self.unit.buff_atk_flat;
                damage += ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64)
                    * (1.0 + ((bonusscaling) as f64).min((i) as f64) * 0.2);
            }
            dps = damage / totalduration * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Carnelian {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Carnelian {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
