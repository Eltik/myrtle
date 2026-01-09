//! Leaderboard route module
//!
//! Provides the `/leaderboard` endpoint for ranking users by various score metrics.

mod handler;
mod types;

pub use handler::get_leaderboard;
pub use types::*;
