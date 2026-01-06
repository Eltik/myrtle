//! DPS calculations for Ling
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Ling operator implementation
pub struct Ling {
    pub unit: OperatorUnit,
}

impl Ling {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("trait", "noDragons", true, &[], &[], 0, 0),
        ("skill", "(Chonker)", false, &[], &[], 0, 0),
        ("trait", "(small)", false, &[0], &[], 0, 0),
        ("talent2", "noTalent2Stacks", true, &[], &[], 2, 0),
    ];

    /// Creates a new Ling operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let mut unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        // Apply init-time modifications from Python __init__
        if unit.module_index == 2
            && unit.module_level == 3
            && (unit.skill_index == 0 || unit.skill_index == 3)
        {
            unit.drone_atk += 60.0;
        }
        if unit.module_index == 2 && unit.module_level == 3 && unit.skill_index == 2 {
            unit.drone_atk += 35.0;
        }
        if unit.module_index == 2 && unit.module_level == 3 && unit.skill_index == 1 {
            unit.drone_atk += 45.0;
        }

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// talentbuff = self.talent2_params[0] * self.talent2_params[2] if self.talent2_dmg else 0
    /// dragons = 2 if self.talent_dmg else 1
    /// if not self.trait_dmg: dragons = 0
    ///
    /// if self.skill == 1:
    /// atkbuff = self.skill_params[0]
    /// aspd = self.skill_params[1]
    ///
    /// final_atk = self.atk * (1+atkbuff + talentbuff + self.buff_atk) + self.buff_atk_flat
    /// final_dragon = self.drone_atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    ///
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmgdrag = np.fmax(final_dragon * (1-res/100), final_dragon * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) + hitdmgdrag/(self.drone_atk_interval/((self.attack_speed + aspd)/100)) * dragons
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + talentbuff + self.buff_atk) + self.buff_atk_flat
    /// final_dragon = self.drone_atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// hitdmgdrag = np.fmax(final_dragon * (1-res/100), final_dragon * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// skilldmgdrag = np.fmax(final_dragon * skill_scale * (1-res/100), final_dragon * skill_scale * 0.05)
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// dpsskill = (skilldmg + dragons * skilldmgdrag) * min(self.targets,2) / sp_cost
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) + hitdmgdrag/(self.drone_atk_interval/(self.attack_speed/100)) * dragons + dpsskill
    /// if self.skill in [0,3]:
    /// atkbuff = self.skill_params[0] * self.skill/3
    /// final_atk = self.atk * (1 + atkbuff + talentbuff + self.buff_atk) + self.buff_atk_flat
    /// chonkerbuff = 0.8 if self.skill_dmg else 0
    /// final_dragon = self.drone_atk * (1+atkbuff + self.buff_atk + chonkerbuff) + self.buff_atk_flat
    /// dragoninterval = self.drone_atk_interval if not self.skill_dmg else 2.3
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// block = 4 if self.skill_dmg else 2
    /// hitdmgdrag = np.fmax(final_dragon - defense, final_dragon * 0.05) * min(self.targets, block)
    /// skilldmg = hitdmg * 0.2
    ///
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) + hitdmgdrag/(dragoninterval/(self.attack_speed/100)) * dragons + skilldmg * 2 * dragons * self.targets
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
        let mut hitdmgdrag: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut final_dragon: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut sp_cost: f64 = 0.0;

        let mut talentbuff = if self.unit.talent2_damage {
            self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0)
                * self.unit.talent2_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut dragons = if self.unit.talent_damage { 2.0 } else { 1.0 };
        if !self.unit.trait_damage {
            dragons = 0.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + atkbuff + talentbuff + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            final_dragon = self.unit.drone_atk * (1.0 + atkbuff + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            hitdmgdrag =
                ((final_dragon * (1.0 - res / 100.0)) as f64).max((final_dragon * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                + hitdmgdrag
                    / ((self.unit.drone_atk_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * dragons;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + talentbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            final_dragon =
                self.unit.drone_atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            hitdmgdrag =
                ((final_dragon * (1.0 - res / 100.0)) as f64).max((final_dragon * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            let mut skilldmgdrag = ((final_dragon * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_dragon * skill_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut dpsskill = (skilldmg + dragons * skilldmgdrag)
                * ((self.unit.targets as f64) as f64).min((2) as f64)
                / sp_cost;
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                + hitdmgdrag
                    / ((self.unit.drone_atk_interval as f64) / (self.unit.attack_speed / 100.0))
                    * dragons
                + dpsskill;
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64)
                / 3.0;
            final_atk = self.unit.atk * (1.0 + atkbuff + talentbuff + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            let mut chonkerbuff = if self.unit.skill_damage { 0.8 } else { 0.0 };
            final_dragon = self.unit.drone_atk * (1.0 + atkbuff + self.unit.buff_atk + chonkerbuff)
                + self.unit.buff_atk_flat;
            let mut dragoninterval = if !self.unit.skill_damage {
                (self.unit.drone_atk_interval as f64)
            } else {
                2.3
            };
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut block = if self.unit.skill_damage { 4.0 } else { 2.0 };
            hitdmgdrag = ((final_dragon - defense) as f64).max((final_dragon * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((block) as f64);
            skilldmg = hitdmg * 0.2;
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                + hitdmgdrag / (dragoninterval / (self.unit.attack_speed / 100.0)) * dragons
                + skilldmg * 2.0 * dragons * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Ling {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ling {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ling {
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
