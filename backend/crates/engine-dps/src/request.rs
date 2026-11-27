use std::sync::Arc;

use myrtle_dps_formulas::{operator_data::OperatorData, operator_unit::OperatorParams};

use crate::zones::BuffZones;

#[derive(Default)]
pub struct DpsRequest {
    pub params: OperatorParams,
    pub zones: Option<BuffZones>,
    pub secondary: Option<SecondaryTarget>,
    /// How `average_dps` cycles the skill. Default `Legacy`.
    pub cycle_model: CycleModel,
    pub clone_operator: Option<Arc<OperatorData>>,
}

impl From<OperatorParams> for DpsRequest {
    fn from(params: OperatorParams) -> Self {
        Self {
            params,
            ..Self::default()
        }
    }
}

/// Enemies hit besides the primary one
#[derive(Debug, Clone, Copy)]
pub struct SecondaryTarget {
    pub defense: f64,
    pub res: f64,
    /// Secondary enemies
    pub count: i32,
}

/// Cycle model for `average_dps`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CycleModel {
    /// Off-skill DPS from `normal_attack`,
    /// SP time `cost / (1 + sp_boost)` for every SP type
    #[default]
    Legacy,
    /// Off-skill DPS from the operator's formula at skill 0 where it has a
    /// skill-0 path, SP time by the skill's SP type
    V2,
}
