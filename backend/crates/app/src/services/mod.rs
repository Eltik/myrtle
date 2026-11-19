//! Use cases grouped by area. A service function takes the handles it needs
//! (`&AppContext` at most) and a `&Subject` when the action is gated.

pub mod accounts;
pub mod catalog;
pub mod community;
pub mod gacha;
pub mod grading;
pub mod i18n;
pub mod improvements;
pub mod leaderboard;
pub mod music;
pub mod planner;
pub mod story;
pub mod sync;
pub mod tier_lists;
