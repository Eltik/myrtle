//! DPS calculations for TerraResearchCommission
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// TerraResearchCommission operator implementation
pub struct TerraResearchCommission {
    pub unit: OperatorUnit,
}

impl TerraResearchCommission {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("talent", "after3BigHits", true, &[], &[], 0, 0)];

    /// Creates a new TerraResearchCommission operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            0, // default_skill_index
            6, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// cdmg = self.talent1_params[4] if self.talent_dmg else 1
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg = np.fmax(final_atk * cdmg - defense, final_atk * cdmg * 0.05)
    /// hitdmg2 = np.fmax(0.5 * final_atk - defense, 0.5 * final_atk * 0.05)
    /// critdmg2 = np.fmax(0.5 * final_atk * cdmg - defense, 0.5 * final_atk * cdmg * 0.05)
    /// avghit = 0.8 * (hitdmg + hitdmg2) + 0.2 * (critdmg + critdmg2)
    /// dps = avghit / self.atk_interval * self.attack_speed / 100 * self.targets
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

        let mut final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut cdmg = if self.unit.talent_damage {
            self.unit.talent1_parameters.get(4).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        let mut critdmg =
            ((final_atk * cdmg - defense) as f64).max((final_atk * cdmg * 0.05) as f64);
        let mut hitdmg2 = ((0.5 * final_atk - defense) as f64).max((0.5 * final_atk * 0.05) as f64);
        let mut critdmg2 =
            ((0.5 * final_atk * cdmg - defense) as f64).max((0.5 * final_atk * cdmg * 0.05) as f64);
        let mut avghit = 0.8 * (hitdmg + hitdmg2) + 0.2 * (critdmg + critdmg2);
        let mut dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
            * (self.unit.targets as f64);
        return dps;
    }
}

impl std::ops::Deref for TerraResearchCommission {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for TerraResearchCommission {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for TerraResearchCommission {
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
