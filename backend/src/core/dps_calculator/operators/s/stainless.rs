//! DPS calculations for Stainless
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Stainless operator implementation
pub struct Stainless {
    pub unit: OperatorUnit,
}

impl Stainless {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Stainless operator
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
    /// if self.skill == 1:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * min(self.targets,2)
    /// if self.skill in [0,3]:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]*self.skill/3) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + self.skill_params[1]*self.skill/3)/100
    /// if not self.skill_dmg: dps = 0
    /// turret_scale = self.params
    /// turret_aoe = self.params2
    /// turrethitdmg = np.fmax(final_atk * turret_scale - defense, final_atk * turret_scale * 0.05)
    /// turretaoedmg = np.fmax(final_atk * turret_aoe - defense, final_atk * turret_aoe * 0.05)
    /// totalturret = turrethitdmg + turretaoedmg * (self.targets - 1)
    /// dps += totalturret * self.hits / 5
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
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        if (self.unit.skill_index as f64) == 1.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 3.0)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 3.0)
                / 100.0;
            if !self.unit.skill_damage {
                dps = 0.0;
            }
            let mut turret_scale = 1.0 /* self.params - needs manual implementation */;
            let mut turret_aoe = 1.0 /* self.params2 - needs manual implementation */;
            let mut turrethitdmg = ((final_atk * turret_scale - defense) as f64)
                .max((final_atk * turret_scale * 0.05) as f64);
            let mut turretaoedmg = ((final_atk * turret_aoe - defense) as f64)
                .max((final_atk * turret_aoe * 0.05) as f64);
            let mut totalturret = turrethitdmg + turretaoedmg * ((self.unit.targets as f64) - 1.0);
            dps += totalturret * 1.0 /* self.hits - needs manual implementation */ / 5.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Stainless {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Stainless {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
