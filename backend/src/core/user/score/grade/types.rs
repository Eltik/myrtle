//! Type definitions for user grading system
//!
//! Provides letter grades (S/A/B/C/D/F) based on normalized score,
//! activity metrics, and engagement depth.

use serde::{Deserialize, Serialize};

/// Letter grade representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
    #[default]
    F,
}

impl Grade {
    /// Convert composite score (0-100) to letter grade
    pub fn from_score(score: f32) -> Self {
        // Clamp to valid range first
        let clamped = score.clamp(0.0, 100.0) as i32;
        match clamped {
            90..=100 => Grade::S,
            75..=89 => Grade::A,
            60..=74 => Grade::B,
            45..=59 => Grade::C,
            30..=44 => Grade::D,
            _ => Grade::F,
        }
    }

    /// Get numeric value for sorting/comparison (higher is better)
    pub fn numeric_value(&self) -> i32 {
        match self {
            Grade::S => 6,
            Grade::A => 5,
            Grade::B => 4,
            Grade::C => 3,
            Grade::D => 2,
            Grade::F => 1,
        }
    }

    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::S => "S",
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }
}

/// Activity metrics breakdown measuring login patterns and consistency
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityMetrics {
    /// Days since last login
    pub days_since_login: i64,
    /// Login recency score (0-100) - based on last_online_ts
    pub login_recency_score: f32,
    /// Login frequency score (0-100) - based on check-in history
    pub login_frequency_score: f32,
    /// Mission completion consistency score (0-100)
    pub consistency_score: f32,
    /// Combined activity score (0-100)
    pub total_activity_score: f32,
    /// Total check-ins recorded
    pub total_check_ins: i32,
    /// Expected check-ins based on account age (capped at 365)
    pub expected_check_ins: i32,
}

/// Engagement depth metrics measuring content variety and progression
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngagementMetrics {
    /// Content variety score (0-100) - how many content types engaged
    pub content_variety_score: f32,
    /// Roguelike engagement depth (0-100)
    pub roguelike_depth_score: f32,
    /// Stage completion diversity (0-100)
    pub stage_diversity_score: f32,
    /// Account progression depth (0-100)
    pub progression_depth_score: f32,
    /// Combined engagement score (0-100)
    pub total_engagement_score: f32,
    /// Number of content types engaged (out of 6)
    pub content_types_engaged: i32,
}

/// Complete grade result with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGrade {
    /// Final letter grade (S/A/B/C/D/F)
    pub grade: Grade,
    /// Composite score (0-100)
    pub composite_score: f32,
    /// Account age in days
    pub account_age_days: i64,
    /// Normalized score component (0-100)
    pub normalized_score: f32,
    /// Activity metrics breakdown
    pub activity_metrics: ActivityMetrics,
    /// Engagement metrics breakdown
    pub engagement_metrics: EngagementMetrics,
    /// Estimated percentile ranking (approximate)
    pub percentile_estimate: f32,
    /// Unix timestamp when grade was calculated
    pub calculated_at: i64,
}

impl Default for UserGrade {
    fn default() -> Self {
        Self {
            grade: Grade::F,
            composite_score: 0.0,
            account_age_days: 0,
            normalized_score: 0.0,
            activity_metrics: ActivityMetrics::default(),
            engagement_metrics: EngagementMetrics::default(),
            percentile_estimate: 0.0,
            calculated_at: 0,
        }
    }
}
