//! DPS calculations for Frostleaf
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Frostleaf operator implementation
pub struct Frostleaf {
    pub unit: OperatorUnit,
}

impl Frostleaf {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[("trait", "rangedAtk", true, &[], &[], 0, 0)];

    /// Creates a new Frostleaf operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atk_scale = 0.8 if not self.trait_dmg else 1
    /// atk_interval = self.atk_interval if self.elite < 2 else self.atk_interval + 0.15
    /// extra_arts_scale = 0.1 if self.module == 1 else 0
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// hitdmgarts = np.fmax(final_atk * extra_arts_scale * (1-res/100), final_atk * extra_arts_scale * 0.05)
    ///
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[2]
    /// skilldmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = (avghit+hitdmgarts)/atk_interval * self.attack_speed/100
    /// if self.skill in [0,2]:
    /// aspd = self.skill_params[1] * self.skill/2
    /// dps = (hitdmg+hitdmgarts) / atk_interval * (self.attack_speed + aspd) / 100
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

        let mut hitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;

        atk_scale = if !self.unit.trait_damage { 0.8 } else { 1.0 };
        atk_interval = if ((self.unit.elite as f64) as f64) < 2.0 {
            (self.unit.attack_interval as f64)
        } else {
            (self.unit.attack_interval as f64) + 0.15
        };
        let mut extra_arts_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        hitdmgarts = ((final_atk * extra_arts_scale * (1.0 - res / 100.0)) as f64)
            .max((final_atk * extra_arts_scale * 0.05) as f64);
        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            skilldmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                    / (((atks_per_skillactivation) as f64).trunc() + 1.0);
            }
            dps = (avghit + hitdmgarts) / atk_interval * self.unit.attack_speed / 100.0;
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64)
                / 2.0;
            dps = (hitdmg + hitdmgarts) / atk_interval * (self.unit.attack_speed + aspd) / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Frostleaf {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Frostleaf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Frostleaf {
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
