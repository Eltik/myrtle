//! DPS calculations for Frost
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Frost operator implementation
pub struct Frost {
    pub unit: OperatorUnit,
}

impl Frost {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("trait", "noMines", true, &[0], &[], 0, 0),
        ("talent", "1MinePerSPcost", true, &[], &[], 0, 0),
        ("skill", "MineInRange", false, &[2], &[], 0, 0),
    ];

    /// Creates a new Frost operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// newdef = np.fmax(0, defense - 40 * self.module_lvl) if self.module == 2 and self.module_lvl > 1 else defense
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.trait_dmg and self.skill > 0:
    /// critdmg = 1.2 if self.module == 2 else 1
    /// mine_scale = self.skill_params[1] if self.skill == 1 else self.skill_params[4]
    /// hitdmg_mine = np.fmax(final_atk * mine_scale - newdef, final_atk * mine_scale * 0.05) * critdmg
    /// if self.skill == 2 and self.skill_dmg:
    /// hitdmg_mine += np.fmax(final_atk * self.skill_params[1] - newdef, final_atk * self.skill_params[1] * 0.05) * 3
    /// hitrate = 5 if self.talent_dmg else max(5, self.skill_cost/(1+self.sp_boost))
    /// dps += hitdmg_mine/hitrate
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

        let mut final_atk: f64 = 0.0;
        let mut hitrate: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut critdmg: f64 = 0.0;

        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut newdef = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            ((0) as f64).max((defense - 40.0 * ((self.unit.module_level as f64) as f64)) as f64)
        } else {
            defense
        };
        hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        if self.unit.trait_damage && (self.unit.skill_index as f64) > 0.0 {
            critdmg = if ((self.unit.module_index as f64) as f64) == 2.0 {
                1.2
            } else {
                1.0
            };
            let mut mine_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                self.unit.skill_parameters.get(4).copied().unwrap_or(0.0)
            };
            let mut hitdmg_mine = ((final_atk * mine_scale - newdef) as f64)
                .max((final_atk * mine_scale * 0.05) as f64)
                * critdmg;
            if (self.unit.skill_index as f64) == 2.0 && self.unit.skill_damage {
                hitdmg_mine += ((final_atk
                    * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                    - newdef) as f64)
                    .max(
                        (final_atk
                            * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                            * 0.05) as f64,
                    )
                    * 3.0;
            }
            hitrate = if self.unit.talent_damage {
                5.0
            } else {
                ((5) as f64).max(
                    ((self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64))) as f64,
                )
            };
            dps += hitdmg_mine / hitrate;
        }
        return dps;
    }
}

impl std::ops::Deref for Frost {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Frost {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Frost {
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
