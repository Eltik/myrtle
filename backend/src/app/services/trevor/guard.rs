//! The master switch and a worker's guard (contract v1.1 sections 2.1, 2.3).

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// `TREVOR_MODE`: `on` (default), `passages` (every ask answered from
/// passages at once, no model job queued), `off` (ask is a 503 "Trevor is
/// resting" unless the answer is cached; panels and cached answers serve).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorMode")]
#[schema(as = TrevorMode)]
pub enum Mode {
    On,
    Passages,
    Off,
}

impl Mode {
    /// The env value; anything unknown reads as `on`, with a warning.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "on" | "" => Some(Self::On),
            "passages" => Some(Self::Passages),
            "off" => Some(Self::Off),
            _ => None,
        }
    }
}

/// A worker's power guard state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorGuardState")]
#[schema(as = TrevorGuardState)]
pub enum GuardState {
    #[default]
    Ok,
    Tripped,
}

/// The heartbeat's `guard` (contract v1.1 section 2.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Guard {
    pub state: GuardState,
    pub reason: Option<String>,
    /// Unix seconds the state began.
    pub since: Option<i64>,
}

/// Online workers per model rung.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS, ToSchema)]
#[ts(export, rename = "TrevorWorkerCounts")]
#[schema(as = TrevorWorkerCounts)]
pub struct WorkerCounts {
    #[ts(type = "number")]
    pub cf: i64,
    #[ts(type = "number")]
    pub cpu: i64,
}
