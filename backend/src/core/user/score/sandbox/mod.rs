//! Sandbox (Reclamation Algorithm) scoring module

pub mod calculate;
pub mod types;

pub use calculate::calculate_sandbox_score;
pub use types::{SandboxAreaScore, SandboxBreakdown, SandboxScore};
