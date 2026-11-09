pub mod engine;
pub mod request;
pub mod shreds;
pub mod supporters;
pub mod zones;

pub use engine::{
    DpsMetrics, DpsResult, OffSkillSource, SkillCategory, SpRecovery, calculate_dps,
    calculate_dps_detailed, calculate_dps_with, calculate_hps, calculate_hps_with,
};
pub use myrtle_dps_formulas::HpsResult;
pub use request::{CycleModel, DpsRequest, SecondaryTarget};

pub const ENGINE_VERSION: &str = env!("DPS_ENGINE_VERSION");
