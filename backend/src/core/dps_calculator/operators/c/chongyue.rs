//! DPS calculations for Chongyue
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Chongyue operator implementation
pub struct Chongyue {
    pub unit: OperatorUnit,
}

impl Chongyue {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("module", ">50%Hp", false, &[], &[2], 0, 0),
        ("talent", "NoSkillCrit", true, &[1], &[], 1, 0),
    ];

    /// Creates a new Chongyue operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// aspd = 10 if self.module == 2 and self.module_dmg else 0
    /// crate = self.talent1_params[0] if self.elite > 0 else 0
    /// dmg = self.talent1_params[1] if self.elite > 0 else 1
    /// duration = self.talent1_params[2] if self.elite > 0 else 0
    ///
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// if self.skill < 2:
    /// if self.talent_dmg and self.elite > 0: skilldmg *= dmg
    /// relevant_hits = int(duration/(self.atk_interval /(self.attack_speed+aspd)*100)) + 1
    /// crit_chance = 1 - (1-crate) ** relevant_hits
    /// hitdmg *= (1-crit_chance) + dmg * crit_chance
    /// dps = (hitdmg + skilldmg/self.skill_cost * self.skill) / self.atk_interval * (self.attack_speed+aspd)/100
    ///
    /// if self.skill == 3:
    /// hits = self.skill_cost // 2 + self.skill_cost % 2
    /// relevant_hits = int(duration/(self.atk_interval /(self.attack_speed+aspd)*100)) * 2 + 2
    /// relevant_hits *= hits/(hits+1) #skill hits cant trigger crit and therefore technically have a lower crit rate than normal attacks, but ehh
    /// crit_chance = 1 - (1-crate) ** relevant_hits
    /// skilldmg *= self.targets
    /// avghit = 2 * (hits * hitdmg + skilldmg) /(hits + 1) * ((1-crit_chance) + dmg * crit_chance)
    /// dps = avghit/self.atk_interval * (self.attack_speed+aspd)/100
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
        let mut skilldmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut relevant_hits: f64 = 0.0;
        let mut crit_chance: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        aspd = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            10.0
        } else {
            0.0
        };
        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut dmg = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut duration = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        skilldmg = ((final_atk * skill_scale - defense) as f64)
            .max((final_atk * skill_scale * 0.05) as f64);
        if (self.unit.skill_index as f64) < 2.0 {
            if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
                skilldmg *= dmg;
            }
            relevant_hits = ((duration
                / ((self.unit.attack_interval as f64) / (self.unit.attack_speed + aspd) * 100.0))
                as f64)
                .trunc()
                + 1.0;
            crit_chance = 1.0 - ((1.0 - crit_rate) as f64).powf(relevant_hits as f64);
            hitdmg *= (1.0 - crit_chance) + dmg * crit_chance;
            dps = (hitdmg
                + skilldmg / (self.unit.skill_cost as f64) * (self.unit.skill_index as f64))
                / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut hits =
                (self.unit.skill_cost as f64) / 2.0 + (self.unit.skill_cost as f64) % 2.0;
            relevant_hits = ((duration
                / ((self.unit.attack_interval as f64) / (self.unit.attack_speed + aspd) * 100.0))
                as f64)
                .trunc()
                * 2.0
                + 2.0;
            relevant_hits *= hits / (hits + 1.0);
            crit_chance = 1.0 - ((1.0 - crit_rate) as f64).powf(relevant_hits as f64);
            skilldmg *= (self.unit.targets as f64);
            avghit = 2.0 * (hits * hitdmg + skilldmg) / (hits + 1.0)
                * ((1.0 - crit_chance) + dmg * crit_chance);
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Chongyue {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Chongyue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Chongyue {
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
