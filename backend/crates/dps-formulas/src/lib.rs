pub mod custom;
pub mod operator_data;
pub mod operator_unit;
pub mod tables;

pub use custom::{dispatch, dispatch_hps, skill_duration_override, skill_duration_override_hps};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HpsResult {
    pub skill_hps: f64,
    pub base_hps: f64,
    pub avg_hps: f64,
}
