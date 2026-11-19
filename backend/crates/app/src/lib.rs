//! L5, use cases: everything `api` and `worker` do goes through a function here
//! that takes an [`AppContext`] (or less) and returns values or an [`AppError`].

pub mod alerts;
pub mod audit;
pub mod base_planner;
pub mod cache;
pub mod config;
pub mod context;
pub mod error;
pub mod limits;
pub mod policy;
pub mod privacy;
pub mod services;
pub mod snapshots;

pub use context::AppContext;
pub use error::AppError;
pub use policy::{Action, Decision, Policy, Resource, Subject};
