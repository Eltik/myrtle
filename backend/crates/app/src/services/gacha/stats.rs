use std::time::Duration;

use myrtle_db::repos::gacha::{self, OperatorPullsRow, SharedTotalsRow};
use serde::{Deserialize, Serialize};

use crate::{AppContext, AppError};

pub const GLOBAL_STATS_KEY: &str = "gacha:global_stats";
pub const PER_BANNER_STATS_KEY: &str = "gacha:per_banner_stats";
pub const SHARED_STATS_PREFIX: &str = "gacha:";
pub const GLOBAL_STATS_TTL: Duration = Duration::from_mins(5);
pub const ENHANCED_STATS_TTL: Duration = Duration::from_mins(10);
pub const PER_BANNER_STATS_TTL: Duration = Duration::from_mins(10);
pub const TOP_N_SIZES: [u32; 3] = [10, 20, 50];
const DAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

#[must_use]
pub fn enhanced_stats_key(top_n: u32, include_timing: bool) -> String {
    format!("gacha:enhanced_stats:{top_n}:{include_timing}")
}

#[must_use]
pub fn snap_top_n(requested: Option<u32>) -> u32 {
    let n = requested.unwrap_or(20);
    TOP_N_SIZES
        .into_iter()
        .find(|&size| n <= size)
        .unwrap_or(TOP_N_SIZES[TOP_N_SIZES.len() - 1])
}

pub async fn flush_shared_stats(ctx: &AppContext) {
    ctx.cache.invalidate_prefix(SHARED_STATS_PREFIX).await;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalGachaStats {
    pub total_pulls: i64,
    pub total_users: i64,
    pub six_star_rate: f64,
    pub five_star_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectiveStats {
    pub total_pulls: i64,
    pub total_users: i64,
    pub total_six_stars: i64,
    pub total_five_stars: i64,
    pub total_four_stars: i64,
    pub total_three_stars: i64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_pull_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PullRates {
    pub six_star_rate: f64,
    pub five_star_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperatorPopularity {
    pub char_id: String,
    pub char_name: String,
    pub rarity: i16,
    pub pull_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HourlyPullData {
    pub hour: i32,
    pub pull_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayOfWeekPullData {
    pub day: i32,
    pub day_name: String,
    pub pull_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatePullData {
    pub date: String,
    pub pull_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PullTimingData {
    pub by_hour: Vec<HourlyPullData>,
    pub by_day_of_week: Vec<DayOfWeekPullData>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_date: Option<Vec<DatePullData>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GachaEnhancedStats {
    pub collective_stats: CollectiveStats,
    pub pull_rates: PullRates,
    pub most_common_operators: Vec<OperatorPopularity>,
    pub average_pulls_to_six_star: f64,
    pub average_pulls_to_five_star: f64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_timing: Option<PullTimingData>,
    pub computed_at: String,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BannerPullStat {
    pub pool_id: String,
    pub pull_count: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub user_count: i64,
}

fn share(part: i64, whole: i64) -> f64 {
    if whole > 0 {
        part as f64 / whole as f64
    } else {
        0.0
    }
}

pub async fn global_stats(ctx: &AppContext) -> Result<GlobalGachaStats, AppError> {
    if let Some(cached) = ctx.cache.get_json(GLOBAL_STATS_KEY).await {
        return Ok(cached);
    }
    let totals = gacha::shared_totals(&mut *ctx.db.conn().await?).await?;
    let result = GlobalGachaStats {
        total_pulls: totals.total_pulls,
        total_users: totals.total_users,
        six_star_rate: share(totals.six_star_count, totals.total_pulls),
        five_star_rate: share(totals.five_star_count, totals.total_pulls),
    };
    ctx.cache
        .set_json(GLOBAL_STATS_KEY, &result, GLOBAL_STATS_TTL)
        .await;
    Ok(result)
}

pub async fn enhanced_stats(
    ctx: &AppContext,
    top_n: Option<u32>,
    include_timing: bool,
) -> Result<GachaEnhancedStats, AppError> {
    let top_n = snap_top_n(top_n);
    let key = enhanced_stats_key(top_n, include_timing);
    if let Some(mut cached) = ctx.cache.get_json::<GachaEnhancedStats>(&key).await {
        cached.cached = true;
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let totals = gacha::shared_totals(&mut conn).await?;
    let operators = gacha::shared_top_operators(&mut conn, i64::from(top_n)).await?;
    let timing = if include_timing {
        let hours = gacha::shared_pulls_by_hour(&mut conn).await?;
        let days = gacha::shared_pulls_by_weekday(&mut conn).await?;
        let dates = gacha::shared_pulls_by_date(&mut conn).await?;
        let total = totals.total_pulls;
        Some(PullTimingData {
            by_hour: hours
                .into_iter()
                .map(|r| HourlyPullData {
                    hour: r.hour,
                    pull_count: r.pull_count,
                    percentage: share(r.pull_count, total),
                })
                .collect(),
            by_day_of_week: days
                .into_iter()
                .map(|r| DayOfWeekPullData {
                    day: r.day,
                    day_name: usize::try_from(r.day)
                        .ok()
                        .and_then(|d| DAY_NAMES.get(d))
                        .copied()
                        .unwrap_or("")
                        .to_owned(),
                    pull_count: r.pull_count,
                    percentage: share(r.pull_count, total),
                })
                .collect(),
            by_date: Some(
                dates
                    .into_iter()
                    .map(|r| DatePullData {
                        date: r.date,
                        pull_count: r.pull_count,
                    })
                    .collect(),
            ),
        })
    } else {
        None
    };
    drop(conn);
    let result = shape_enhanced(&totals, operators, timing);
    ctx.cache.set_json(&key, &result, ENHANCED_STATS_TTL).await;
    Ok(result)
}

#[must_use]
pub fn shape_enhanced(
    totals: &SharedTotalsRow,
    operators: Vec<OperatorPullsRow>,
    pull_timing: Option<PullTimingData>,
) -> GachaEnhancedStats {
    let total = totals.total_pulls;
    let per = |count: i64| {
        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        }
    };
    GachaEnhancedStats {
        collective_stats: CollectiveStats {
            total_pulls: total,
            total_users: totals.total_users,
            total_six_stars: totals.six_star_count,
            total_five_stars: totals.five_star_count,
            total_four_stars: totals.four_star_count,
            total_three_stars: totals.three_star_count,
            first_pull_at: totals.first_pull_at,
        },
        pull_rates: PullRates {
            six_star_rate: share(totals.six_star_count, total),
            five_star_rate: share(totals.five_star_count, total),
        },
        most_common_operators: operators
            .into_iter()
            .map(|r| OperatorPopularity {
                char_id: r.char_id,
                char_name: String::new(),
                rarity: r.rarity,
                pull_count: r.pull_count,
                percentage: share(r.pull_count, total),
            })
            .collect(),
        average_pulls_to_six_star: per(totals.six_star_count),
        average_pulls_to_five_star: per(totals.five_star_count),
        pull_timing,
        computed_at: chrono::Utc::now().to_rfc3339(),
        cached: false,
    }
}

pub async fn per_banner_stats(ctx: &AppContext) -> Result<Vec<BannerPullStat>, AppError> {
    if let Some(cached) = ctx.cache.get_json(PER_BANNER_STATS_KEY).await {
        return Ok(cached);
    }
    let rows = gacha::shared_banner_totals(&mut *ctx.db.conn().await?).await?;
    let result: Vec<BannerPullStat> = rows
        .into_iter()
        .map(|r| BannerPullStat {
            pool_id: r.pool_id,
            pull_count: r.pull_count,
            six_star_count: r.six_star_count,
            five_star_count: r.five_star_count,
            user_count: r.user_count,
        })
        .collect();
    ctx.cache
        .set_json(PER_BANNER_STATS_KEY, &result, PER_BANNER_STATS_TTL)
        .await;
    Ok(result)
}
