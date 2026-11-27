pub mod custom;
pub mod operator_data;
pub mod operator_unit;
pub mod tables;

#[cfg(test)]
#[path = "../tests/unit/mod.rs"]
mod unit;

pub use custom::{dispatch, dispatch_hps, skill_duration_override, skill_duration_override_hps};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HpsResult {
    pub skill_hps: f64,
    pub base_hps: f64,
    pub avg_hps: f64,
}
