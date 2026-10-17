//! The worker side, behind the scoped worker key (contract section 3): claim
//! a job (long-poll), heartbeat, stream deltas, post the result, publish a
//! corpus version; and the stale sweep that runs on its own tick.
//!
//! - [`auth`]: the scoped worker key and the extractor on every worker route.
//! - [`wire`]: the worker-facing request and response bodies.
//! - [`queue`]: heartbeat, claim, deltas and the stale sweep.
//! - [`result`]: storing a worker's result.
//! - [`publish`]: publishing a corpus version.
//! - [`warm`]: the warm-up list and its jobs after a publish.

pub mod auth;
pub mod publish;
pub mod queue;
pub mod result;
pub mod warm;
pub mod wire;

pub use auth::*;
pub use publish::*;
pub use queue::*;
pub use result::*;
pub use warm::*;
pub use wire::*;
