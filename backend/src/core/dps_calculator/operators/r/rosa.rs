//! DPS calculations for Rosa
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Rosa operator implementation
pub struct Rosa {
    pub unit: OperatorUnit,
}

impl Rosa {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Rosa operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// atkbuff = self.talent2_params[0]
    /// atk_scale = 1
    /// additional_scale = 0
    /// defshred = 0
    /// if self.talent_dmg: #aka: if heavy
    /// if self.elite > 0: defshred = 0.2 + 0.2 * self.elite
    /// if self.module == 1:
    /// atk_scale = 1.15
    /// if self.module_lvl == 2: additional_scale = 0.4
    /// if self.module_lvl == 3: additional_scale = 0.6
    /// newdef = defense * (1-defshred)
    ///
    /// if self.skill < 2:
    /// atkbuff += self.skill_params[0] * self.skill
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// extradmg = np.fmax(final_atk * atk_scale * additional_scale - newdef, final_atk * atk_scale * additional_scale * 0.05)
    /// dps = (hitdmg+extradmg)/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// extradmg = np.fmax(final_atk * atk_scale * additional_scale - newdef, final_atk * atk_scale * additional_scale * 0.05)
    /// dps = (hitdmg+extradmg)/self.atk_interval * self.attack_speed/100 * min(self.targets,2)
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[2]
    /// maxtargets = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// extradmg = np.fmax(final_atk * atk_scale * additional_scale - newdef, final_atk * atk_scale * additional_scale * 0.05)
    /// dps = (hitdmg+extradmg) * min(self.targets,maxtargets)
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
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut extradmg: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        atkbuff = self.unit.talent2_parameters[0];
        atk_scale = 1.0;
        let mut additional_scale = 0.0;
        defshred = 0.0;
        if self.unit.talent_damage {
            // aka: if heavy
            if (self.unit.elite as f64) > 0.0 {
                defshred = 0.2 + 0.2 * (self.unit.elite as f64);
            }
            if (self.unit.module_index as f64) == 1.0 {
                atk_scale = 1.15;
                if (self.unit.module_level as f64) == 2.0 {
                    additional_scale = 0.4;
                }
                if (self.unit.module_level as f64) == 3.0 {
                    additional_scale = 0.6;
                }
            }
        }
        let mut newdef = defense * (1.0 - defshred);
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff += self.unit.skill_parameters[0] * (self.unit.skill_index as f64);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            extradmg = ((final_atk * atk_scale * additional_scale - newdef) as f64)
                .max((final_atk * atk_scale * additional_scale * 0.05) as f64);
            dps = (hitdmg + extradmg) / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters[0];
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            extradmg = ((final_atk * atk_scale * additional_scale - newdef) as f64)
                .max((final_atk * atk_scale * additional_scale * 0.05) as f64);
            dps = (hitdmg + extradmg) / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters[2];
            let mut maxtargets = self.unit.skill_parameters[0];
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            extradmg = ((final_atk * atk_scale * additional_scale - newdef) as f64)
                .max((final_atk * atk_scale * additional_scale * 0.05) as f64);
            dps =
                (hitdmg + extradmg) * ((self.unit.targets as f64) as f64).min((maxtargets) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Rosa {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Rosa {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
