//! DPS Calculator API routes
//!
//! Provides endpoints for calculating operator DPS against enemy stats.

mod calculate;
mod list;

pub use calculate::calculate_dps;
pub use list::list_operators;
