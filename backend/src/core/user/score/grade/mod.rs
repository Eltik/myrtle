//! User grading system
//!
//! Calculates letter grades (S/A/B/C/D/F) based on:
//! - Normalized total score relative to account age (50%)
//! - Activity metrics: login frequency and recency (30%)
//! - Engagement depth: content variety and progression (20%)
//!
//! Grades are calculated in real-time from live server data.

pub mod calculate;
pub mod types;

pub use calculate::calculate_user_grade;
pub use types::{ActivityMetrics, EngagementMetrics, Grade, UserGrade};
