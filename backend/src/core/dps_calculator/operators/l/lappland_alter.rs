//! DPS calculations for LapplandAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// LapplandAlter operator implementation
pub struct LapplandAlter {
    pub unit: OperatorUnit,
}

impl LapplandAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "noTalentBuffs", true, &[], &[], 1, 0),
        ("trait", "minDroneDmg", true, &[], &[], 0, 0),
    ];

    /// Creates a new LapplandAlter operator
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
    /// drone_dmg = 1.1
    /// drones = 1
    /// if not self.trait_dmg:
    /// drone_dmg = 0.35 if self.module == 1 else 0.2
    /// if self.talent_dmg and self.elite > 0:
    /// drone_dmg *= self.talent1_params[1]
    /// drones += 1
    /// try: aspd = self.talent2_params[1]
    /// except: aspd = 0
    /// atkbuff = self.skill_params[0] * min(self.skill,1)
    /// if self.skill == 1: drones += 1
    /// if self.skill == 2: drones += 3
    /// if self.skill == 3: drones += 2
    /// final_atk = self.atk * (1+atkbuff+self.buff_atk) + self.buff_atk_flat
    /// drone_atk = drone_dmg * final_atk
    /// dmgperinterval = final_atk + drones * drone_atk
    /// hitdmgarts = np.fmax(dmgperinterval *(1-res/100), dmgperinterval * 0.05)
    /// dps = hitdmgarts/self.atk_interval*(self.attack_speed+aspd)/100
    /// if self.skill == 3:
    /// dps += self.targets * final_atk * self.skill_params[4] * np.fmax((1-res/100),0.05)
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

        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut drone_dmg = 1.1;
        let mut drones = 1.0;
        if !self.unit.trait_damage {
            drone_dmg = if ((self.unit.module_index as f64) as f64) == 1.0 {
                0.35
            } else {
                0.2
            };
        }
        if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
            drone_dmg *= self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
            drones += 1.0;
        }
        let mut aspd = self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0); // try-except fallback
        let mut atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
            * ((self.unit.skill_index as f64) as f64).min((1) as f64);
        if (self.unit.skill_index as f64) == 1.0 {
            drones += 1.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            drones += 3.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            drones += 2.0;
        }
        let mut final_atk =
            self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut drone_atk = drone_dmg * final_atk;
        let mut dmgperinterval = final_atk + drones * drone_atk;
        let mut hitdmgarts =
            ((dmgperinterval * (1.0 - res / 100.0)) as f64).max((dmgperinterval * 0.05) as f64);
        let mut dps = hitdmgarts / (self.unit.attack_interval as f64)
            * (self.unit.attack_speed + aspd)
            / 100.0;
        if (self.unit.skill_index as f64) == 3.0 {
            dps += (self.unit.targets as f64)
                * final_atk
                * self.unit.skill_parameters.get(4).copied().unwrap_or(0.0)
                * ((1.0 - res / 100.0) as f64).max((0.05) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for LapplandAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for LapplandAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for LapplandAlter {
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
