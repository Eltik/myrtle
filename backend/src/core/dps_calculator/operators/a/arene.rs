//! DPS calculations for Arene
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Arene operator implementation
pub struct Arene {
    pub unit: OperatorUnit,
}

impl Arene {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("trait", "rangedAtk", true, &[1], &[], 0, 0),
        ("talent", "vsDrones", false, &[], &[], 1, 0),
        ("module", "+12aspd(mod)", false, &[], &[2], 0, 0),
    ];

    /// Creates a new Arene operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atk_scale = self.talent1_params[0] if self.talent_dmg else 1
    /// aspd = 12 if self.module == 2 and (self.targets > 1 or self.module_dmg) else 0
    /// if not self.trait_dmg and self.skill != 2: atk_scale *= 0.8
    ///
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1+ self.buff_atk) + self.buff_atk_flat
    /// if self.skill == 0:
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 1:
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05)
    /// dps = 2 * hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// hitdmgarts = np.fmax(final_atk * skill_scale * atk_scale  * (1-res/100), final_atk * skill_scale * atk_scale * 0.05)
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100 * min(2,self.targets)
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

        // Arene: trait_dmg is false when skill == 1 and talent_dmg (Python __init__ logic)
        let trait_damage = if self.unit.skill_index == 1 && self.unit.talent_damage {
            false
        } else {
            self.unit.trait_damage
        };

        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;

        atk_scale = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        aspd = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.targets as f64) > 1.0 || self.unit.module_damage)
        {
            12.0
        } else {
            0.0
        };
        if !trait_damage && (self.unit.skill_index as f64) != 2.0 {
            atk_scale *= 0.8;
        }
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) == 0.0 {
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            hitdmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = 2.0 * hitdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            hitdmgarts = ((final_atk * skill_scale * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            dps = hitdmgarts / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((2) as f64).min((self.unit.targets as f64) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Arene {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Arene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Arene {
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
