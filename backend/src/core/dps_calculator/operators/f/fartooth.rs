//! DPS calculations for Fartooth
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Fartooth operator implementation
pub struct Fartooth {
    pub unit: OperatorUnit,
}

impl Fartooth {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Fartooth operator
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
    /// atkbuff = 0
    /// aspd = 0
    /// atk_scale = 1.15 if self.module == 1 and self.module_dmg else 1
    /// #talent/module buffs
    /// atkbuff += self.talent1_params[0]
    /// try:
    /// aspd += self.talent1_params[2]
    /// except:
    /// pass
    /// if self.skill == 1:
    /// atkbuff += self.skill_params[0]
    /// aspd += self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// if self.skill in [0,2]:
    /// aspd += self.skill_params[0] * self.skill/2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[0]
    /// dmgscale = 1
    /// if self.skill_dmg:
    /// dmgscale = self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)*dmgscale
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100))
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
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut dmgscale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;

        atkbuff = 0.0;
        aspd = 0.0;
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        // talent/module buffs
        atkbuff += self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        // UNTRANSLATED: try:
        aspd += self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0);
        // UNTRANSLATED: pass
        if (self.unit.skill_index as f64) == 1.0 {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            aspd += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            aspd += self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64)
                / 2.0;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            dmgscale = 1.0;
            if self.unit.skill_damage {
                dmgscale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            }
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * dmgscale;
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        return dps;
    }
}

impl std::ops::Deref for Fartooth {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Fartooth {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Fartooth {
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
