pub mod progress;
pub mod requirements;
pub mod types;
pub mod validation;

pub use progress::{calculate_leveling_costs, raise_to_roster, stored_plan_met};
pub use requirements::{
    MAX_FLATTEN_TIER, MIN_FLATTEN_TIER, PlannerCtx, calculate_requirements,
    requirements_by_operator,
};
pub use types::*;
pub use validation::{ensure_plannable_id, module_phase_to_int, validate_plan, validate_preset};

pub const LMD_ITEM: &str = "4001";
pub const EXP_ITEM: &str = "5001";

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("{0}")]
    Invalid(String),
    #[error("plan targets did not encode: {0}")]
    Encode(#[from] serde_json::Error),
}
