//! DPS calculations for Ascalon
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Ascalon operator implementation
pub struct Ascalon {
    pub unit: OperatorUnit,
}

impl Ascalon {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "1Stack", true, &[], &[], 0, 0),
        ("talent2", "NoRangedTiles", true, &[], &[], 0, 0),
    ];

    /// Creates a new Ascalon operator
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
    /// talentstacks = 3 if self.talent_dmg else 1
    /// talentscale = self.talent1_params[1]
    /// aspd = self.talent2_params[0]
    /// if self.elite == 2 and self.talent2_dmg: aspd += self.talent2_params[1]
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * 2
    /// sp_cost = self.skill_cost + 1.2
    /// atkcycle = self.atk_interval/(self.attack_speed+ aspd)*100
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// dps = avghit/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// if self.skill in [0,2]:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill/2) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// if self.skill == 3:
    /// atk_interval = self.atk_interval + self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[1]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// dps += self.targets * final_atk * talentstacks * talentscale * np.fmax(1-res/100, 0.05)
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
        let mut skill_scale: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        let mut talentstacks = if self.unit.talent_damage { 3.0 } else { 1.0 };
        let mut talentscale = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        aspd = self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0);
        if (self.unit.elite as f64) == 2.0 && self.unit.talent2_damage {
            aspd += self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * 2.0;
            sp_cost = (self.unit.skill_cost as f64) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed + aspd) * 100.0;
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                    / atks_per_skillactivation;
            }
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 2.0)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64)
                + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / atk_interval * (self.unit.attack_speed + aspd) / 100.0
                * (self.unit.targets as f64);
        }
        dps += (self.unit.targets as f64)
            * final_atk
            * talentstacks
            * talentscale
            * ((1.0 - res / 100.0) as f64).max((0.05) as f64);
        return dps;
    }
}

impl std::ops::Deref for Ascalon {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Ascalon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Ascalon {
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
