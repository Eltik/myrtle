//! DPS calculations for LuoXiaohei
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// LuoXiaohei operator implementation
pub struct LuoXiaohei {
    pub unit: OperatorUnit,
}

impl LuoXiaohei {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new LuoXiaohei operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        // Apply init-time modifications from Python __init__

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// dmg_scale = 1 + 0.04 * self.module_lvl if self.below50 else 1
    /// aspd = 12 if self.module == 2 and (self.module_dmg or self.targets > 1) else 0
    ///
    /// if self.skill == 0:
    /// atk_scale = 1 if self.trait_dmg else 0.8
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk *atk_scale - defense, final_atk * atk_scale * 0.05) * dmg_scale
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100
    /// if self.skill == 1:
    /// aspd += self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * dmg_scale
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100 * min(self.targets,2)
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * dmg_scale
    /// newdef = np.fmax(defense - self.skill_params[2], 0)
    /// hitdmg2 = np.fmax(final_atk - newdef, final_atk * 0.05) * dmg_scale
    /// if self.below50:
    /// hitdmg += hitdmg2
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd)/ 100 * min(self.targets,2)
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

        // Calculate below50 from __init__ logic
        let mut below50 = false;
        below50 = false;
        if self.unit.skill_index == 2 && self.unit.skill_damage {
            below50 = true;
        }
        if self.unit.module_index == 2 && self.unit.module_level > 1 && self.unit.talent_damage {
            below50 = true;
        }

        let mut hitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;

        let mut dmg_scale = if below50 {
            1.0 + 0.04 * ((self.unit.module_level as f64) as f64)
        } else {
            1.0
        };
        aspd = if ((self.unit.module_index as f64) as f64) == 2.0
            && (self.unit.module_damage || (self.unit.targets as f64) > 1.0)
        {
            12.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            atk_scale = if self.unit.trait_damage { 1.0 } else { 0.8 };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmg_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            aspd += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * dmg_scale;
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * dmg_scale;
            let mut newdef = ((defense - self.unit.skill_parameters.get(2).copied().unwrap_or(0.0))
                as f64)
                .max((0) as f64);
            let mut hitdmg2 =
                ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64) * dmg_scale;
            if below50 {
                hitdmg += hitdmg2;
            }
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for LuoXiaohei {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for LuoXiaohei {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for LuoXiaohei {
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
