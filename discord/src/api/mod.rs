use std::time::Duration;

pub mod gamedata;
pub mod operator_detail;
pub mod stats;
pub mod status;

pub const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);
