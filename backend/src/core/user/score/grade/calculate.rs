//! Grade calculation logic
//!
//! Calculates user grades from score and activity metrics using live server data.

use crate::core::user::types::User;

use super::super::types::{ScoreBreakdown, UserScore};
use super::types::{ActivityMetrics, EngagementMetrics, Grade, UserGrade};

/// Constants for grade calculation
mod constants {
    /// Expected score progression per day of account age
    pub const BASE_SCORE_PER_DAY: f32 = 15.0;
    /// Theoretical maximum score for a fully developed account
    pub const MAX_EXPECTED_SCORE: f32 = 150_000.0;

    // Weight distribution for composite score
    /// Weight for normalized total score component
    pub const SCORE_WEIGHT: f32 = 0.50;
    /// Weight for activity metrics component
    pub const ACTIVITY_WEIGHT: f32 = 0.30;
    /// Weight for engagement depth component
    pub const ENGAGEMENT_WEIGHT: f32 = 0.20;

    // Activity sub-weights
    /// Weight for login recency within activity score
    pub const RECENCY_WEIGHT: f32 = 0.40;
    /// Weight for login frequency within activity score
    pub const FREQUENCY_WEIGHT: f32 = 0.40;
    /// Weight for consistency within activity score
    pub const CONSISTENCY_WEIGHT: f32 = 0.20;

    // Engagement sub-weights
    /// Weight for content variety within engagement score
    pub const VARIETY_WEIGHT: f32 = 0.25;
    /// Weight for roguelike depth within engagement score
    pub const ROGUELIKE_WEIGHT: f32 = 0.25;
    /// Weight for stage diversity within engagement score
    pub const STAGE_WEIGHT: f32 = 0.25;
    /// Weight for progression depth within engagement score
    pub const PROGRESSION_WEIGHT: f32 = 0.25;
}

/// Calculate user grade from score and user data
///
/// Uses the existing UserScore (which contains total_score and breakdown)
/// along with live user data (timestamps, check-ins, missions) to compute
/// a comprehensive grade.
///
/// The grade is calculated as of the user's last login time (last_online_ts),
/// not the current time. This ensures the grade reflects the user's state
/// at the time of the data snapshot.
pub fn calculate_user_grade(user: &User, score: &UserScore) -> UserGrade {
    // Use the user's last login time as the reference point for calculations
    // This ensures the grade reflects their state at the time of the data snapshot
    let reference_time = user.status.last_online_ts;
    let account_age_days = calculate_account_age_days(user.status.register_ts, reference_time);

    // Calculate each component
    let normalized_score = normalize_total_score(score.total_score, account_age_days);
    let activity_metrics = calculate_activity_metrics(user, reference_time, account_age_days);
    let engagement_metrics = calculate_engagement_metrics(&score.breakdown);

    // Compute composite score using weights
    let composite_score = (normalized_score * constants::SCORE_WEIGHT)
        + (activity_metrics.total_activity_score * constants::ACTIVITY_WEIGHT)
        + (engagement_metrics.total_engagement_score * constants::ENGAGEMENT_WEIGHT);

    // Clamp to valid range
    let composite_score = composite_score.clamp(0.0, 100.0);

    // Determine letter grade
    let grade = Grade::from_score(composite_score);

    // Estimate percentile
    let percentile_estimate = estimate_percentile(composite_score);

    UserGrade {
        grade,
        composite_score,
        account_age_days,
        normalized_score,
        activity_metrics,
        engagement_metrics,
        percentile_estimate,
        calculated_at: reference_time,
    }
}

/// Calculate account age in days from registration timestamp
fn calculate_account_age_days(register_ts: i64, now: i64) -> i64 {
    let age_seconds = now - register_ts;
    if age_seconds < 0 {
        return 0;
    }
    age_seconds / 86400 // seconds per day
}

/// Normalize total score relative to account age
///
/// Newer accounts are evaluated relative to expected progression,
/// while older accounts are evaluated more on absolute score.
fn normalize_total_score(total_score: f32, account_age_days: i64) -> f32 {
    // Handle edge case of very new accounts
    if account_age_days <= 0 {
        return 0.0;
    }

    // Calculate expected score for this account age
    let expected_score = (account_age_days as f32 * constants::BASE_SCORE_PER_DAY)
        .min(constants::MAX_EXPECTED_SCORE);

    // Score ratio: how well they've progressed relative to time
    let score_ratio = if expected_score > 0.0 {
        (total_score / expected_score).min(2.0) // Cap at 2x expected
    } else {
        0.0
    };

    // Absolute factor: raw progress toward theoretical maximum
    let absolute_factor = (total_score / constants::MAX_EXPECTED_SCORE).min(1.0);

    // Weight shifts from relative to absolute as account ages
    // At 0 days: 100% relative, at 365+ days: 50% relative / 50% absolute
    let age_weight = (account_age_days as f32 / 365.0).min(1.0);

    let relative_component = score_ratio * 50.0 * (1.0 - age_weight * 0.5);
    let absolute_component = absolute_factor * 100.0 * age_weight * 0.5;

    (relative_component + absolute_component).min(100.0)
}

/// Calculate activity metrics from user data
fn calculate_activity_metrics(user: &User, now: i64, _account_age_days: i64) -> ActivityMetrics {
    let days_since_login = calculate_days_since_login(user.status.last_online_ts, now);

    // 1. Login Recency Score - exponential decay based on last login
    let login_recency_score = calculate_recency_score(days_since_login);

    // 2. Login Frequency Score - based on check-in cycle completion
    // FIX: Count actual check-ins (1s in the array), not the array length
    let check_ins_this_cycle = user
        .check_in
        .check_in_history
        .iter()
        .filter(|&&x| x == 1)
        .count() as i32;
    let check_in_cycle_length = user.check_in.check_in_history.len() as i32;

    // Calculate completion rate for current cycle
    let check_in_completion_rate = if check_in_cycle_length > 0 {
        (check_ins_this_cycle as f32 / check_in_cycle_length as f32) * 100.0
    } else {
        0.0
    };
    let login_frequency_score = check_in_completion_rate;

    // 3. Mission Consistency Score - based on daily/weekly mission states
    let consistency_score = calculate_consistency_score(user);

    // Combined activity score
    let total_activity_score = (login_recency_score * constants::RECENCY_WEIGHT)
        + (login_frequency_score * constants::FREQUENCY_WEIGHT)
        + (consistency_score * constants::CONSISTENCY_WEIGHT);

    ActivityMetrics {
        days_since_login,
        login_recency_score,
        login_frequency_score,
        consistency_score,
        total_activity_score,
        check_ins_this_cycle,
        check_in_cycle_length,
        check_in_completion_rate,
    }
}

/// Calculate days since last login
fn calculate_days_since_login(last_online_ts: i64, now: i64) -> i64 {
    let diff = now - last_online_ts;
    if diff < 0 {
        return 0;
    }
    diff / 86400
}

/// Calculate recency score based on days since last login
/// Uses step function with exponential decay pattern
fn calculate_recency_score(days_since_login: i64) -> f32 {
    match days_since_login {
        0 => 100.0,      // Logged in today
        1 => 95.0,       // Yesterday
        2..=3 => 85.0,   // 2-3 days ago
        4..=7 => 70.0,   // Within a week
        8..=14 => 50.0,  // Within two weeks
        15..=30 => 30.0, // Within a month
        31..=60 => 15.0, // Within two months
        61..=90 => 8.0,  // Within three months
        _ => 3.0,        // Very inactive
    }
}

/// Calculate consistency score based on mission completion
/// Checks daily and weekly mission states (state 3 = completed and claimed)
fn calculate_consistency_score(user: &User) -> f32 {
    let daily_completed = count_completed_missions(&user.mission.missions.daily);
    let weekly_completed = count_completed_missions(&user.mission.missions.weekly);

    // Typical active player completes ~12 daily missions and ~8 weekly missions
    let total_completed = daily_completed + weekly_completed;
    let max_expected = 20;

    ((total_completed as f32 / max_expected as f32) * 100.0).min(100.0)
}

/// Count missions with state >= 2 (completed)
fn count_completed_missions(
    missions: &std::collections::HashMap<String, crate::core::user::types::MissionData>,
) -> i32 {
    missions.values().filter(|m| m.state >= 2).count() as i32
}

/// Calculate engagement metrics from score breakdown
fn calculate_engagement_metrics(breakdown: &ScoreBreakdown) -> EngagementMetrics {
    // 1. Content Variety Score - how many different content types engaged
    let content_types_engaged = count_content_types_engaged(breakdown);
    let content_variety_score = (content_types_engaged as f32 / 6.0) * 100.0;

    // 2. Roguelike Depth Score
    let roguelike_depth_score = calculate_roguelike_engagement(breakdown);

    // 3. Stage Diversity Score
    let stage_diversity_score = calculate_stage_diversity(breakdown);

    // 4. Progression Depth Score
    let progression_depth_score = calculate_progression_depth(breakdown);

    // Combined engagement score
    let total_engagement_score = (content_variety_score * constants::VARIETY_WEIGHT)
        + (roguelike_depth_score * constants::ROGUELIKE_WEIGHT)
        + (stage_diversity_score * constants::STAGE_WEIGHT)
        + (progression_depth_score * constants::PROGRESSION_WEIGHT);

    EngagementMetrics {
        content_variety_score,
        roguelike_depth_score,
        stage_diversity_score,
        progression_depth_score,
        total_engagement_score,
        content_types_engaged,
    }
}

/// Count how many different content types the user has engaged with
/// Uses percentage thresholds for consistent measurement
fn count_content_types_engaged(breakdown: &ScoreBreakdown) -> i32 {
    let mut count = 0;

    // Roguelike engagement (>5% themes completion)
    if breakdown.roguelike_themes_completion_percentage > 5.0 {
        count += 1;
    }

    // Sandbox engagement (>5% completion)
    if breakdown.sandbox_completion_percentage > 5.0 {
        count += 1;
    }

    // Medal collection (>5% completion)
    if breakdown.medal_completion_percentage > 5.0 {
        count += 1;
    }

    // Mainline progression (>50% completion)
    if breakdown.mainline_completion > 50.0 {
        count += 1;
    }

    // Stage completion (>10% overall completion)
    if breakdown.overall_stage_completion_percentage > 10.0 {
        count += 1;
    }

    // Base building
    if breakdown.base_max_level_buildings > 0 {
        count += 1;
    }

    count
}

/// Calculate roguelike engagement depth score using completion percentages
fn calculate_roguelike_engagement(breakdown: &ScoreBreakdown) -> f32 {
    // Themes completion factor (20%) - uses game data max
    let themes_factor = breakdown.roguelike_themes_completion_percentage * 0.20;

    // Collectibles completion factor (40%) - actual completable content
    let collectibles_factor = breakdown.roguelike_collectibles_completion_percentage * 0.40;

    // Challenge completion factor (40%) - grade 2 clears as % of total challenges
    let challenge_factor = breakdown.roguelike_challenges_completion_percentage * 0.40;

    themes_factor + collectibles_factor + challenge_factor
}

/// Calculate stage completion diversity score
fn calculate_stage_diversity(breakdown: &ScoreBreakdown) -> f32 {
    // Balance between mainline, sidestory, and activity completion
    let mainline_weight = breakdown.mainline_completion * 0.40;
    let sidestory_weight = breakdown.sidestory_completion * 0.30;
    let activity_weight = breakdown.activity_completion * 0.30;

    mainline_weight + sidestory_weight + activity_weight
}

/// Calculate account progression depth score using completion percentages
fn calculate_progression_depth(breakdown: &ScoreBreakdown) -> f32 {
    // Operator collection factor (30%) - owned / available
    let operator_factor = breakdown.operator_collection_percentage * 0.30;

    // Operator investment factor (30%) - E2 count as ratio of owned operators
    let investment_factor = if breakdown.total_operators > 0 {
        ((breakdown.e2_count as f32 / breakdown.total_operators as f32) * 100.0).min(100.0) * 0.30
    } else {
        0.0
    };

    // Medal completion factor (40%)
    let medal_factor = breakdown.medal_completion_percentage * 0.40;

    operator_factor + investment_factor + medal_factor
}

/// Estimate percentile ranking based on composite score
/// This is an approximation based on expected score distribution
fn estimate_percentile(composite_score: f32) -> f32 {
    match composite_score as i32 {
        95..=100 => 99.0,
        90..=94 => 95.0,
        85..=89 => 90.0,
        80..=84 => 80.0,
        75..=79 => 70.0,
        70..=74 => 60.0,
        60..=69 => 50.0,
        50..=59 => 40.0,
        40..=49 => 30.0,
        30..=39 => 20.0,
        20..=29 => 10.0,
        _ => 5.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_from_score() {
        assert_eq!(Grade::from_score(95.0), Grade::S);
        assert_eq!(Grade::from_score(90.0), Grade::S);
        assert_eq!(Grade::from_score(89.0), Grade::A);
        assert_eq!(Grade::from_score(75.0), Grade::A);
        assert_eq!(Grade::from_score(74.0), Grade::B);
        assert_eq!(Grade::from_score(60.0), Grade::B);
        assert_eq!(Grade::from_score(59.0), Grade::C);
        assert_eq!(Grade::from_score(45.0), Grade::C);
        assert_eq!(Grade::from_score(44.0), Grade::D);
        assert_eq!(Grade::from_score(30.0), Grade::D);
        assert_eq!(Grade::from_score(29.0), Grade::F);
        assert_eq!(Grade::from_score(0.0), Grade::F);
    }

    #[test]
    fn test_recency_score() {
        assert_eq!(calculate_recency_score(0), 100.0);
        assert_eq!(calculate_recency_score(1), 95.0);
        assert_eq!(calculate_recency_score(3), 85.0);
        assert_eq!(calculate_recency_score(7), 70.0);
        assert_eq!(calculate_recency_score(14), 50.0);
        assert_eq!(calculate_recency_score(30), 30.0);
        assert_eq!(calculate_recency_score(60), 15.0);
        assert_eq!(calculate_recency_score(100), 3.0);
    }

    #[test]
    fn test_account_age_calculation() {
        let now = 1704825600; // 2024-01-10
        let register_ts = 1672531200; // 2023-01-01
        let age = calculate_account_age_days(register_ts, now);
        assert_eq!(age, 373); // 373 days between these timestamps
    }

    #[test]
    fn test_normalize_score_new_account() {
        // New account (7 days) with 100 score
        let normalized = normalize_total_score(100.0, 7);
        // Expected score at 7 days = 7 * 15 = 105
        // Score ratio = 100/105 ≈ 0.95
        // Age weight = 7/365 ≈ 0.019
        // Should be mostly relative scoring
        assert!(normalized > 40.0 && normalized < 60.0);
    }

    #[test]
    fn test_normalize_score_mature_account() {
        // 1 year old account with 50000 score
        let normalized = normalize_total_score(50000.0, 365);
        // Should factor in both relative and absolute
        assert!(normalized > 30.0 && normalized < 70.0);
    }
}
