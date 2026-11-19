//! Job bodies: trending windows, the daily leaderboard snapshot, community ownership aggregates.

use chrono::{DateTime, Utc};
use myrtle_db::repos::{leaderboard, stats, tier_lists};

use crate::{AppContext, AppError};

fn same_utc_day(last: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    last.is_some_and(|last| last.date_naive() == now.date_naive())
}

/// `JobKind::TrendingRecompute`
pub async fn recompute_trending(ctx: &AppContext) -> Result<String, AppError> {
    let top_n = ctx.config.jobs.trending.top_n;
    let mut conn = ctx.db.conn().await?;
    tier_lists::recompute_trending(&mut conn, top_n).await?;
    Ok(format!(
        "recomputed trending windows (top {top_n} per list)"
    ))
}

/// `JobKind::LeaderboardSnapshot`: at most one a UTC day unless `force` (a manual refresh)
pub async fn snapshot_leaderboard(ctx: &AppContext, force: bool) -> Result<String, AppError> {
    let mut conn = ctx.db.conn().await?;
    if !force
        && same_utc_day(
            leaderboard::latest_snapshot_at(&mut conn).await?,
            Utc::now(),
        )
    {
        return Ok("already snapshotted today".to_owned());
    }
    let id = leaderboard::take_snapshot(&mut conn).await?;
    Ok(format!("snapshot {id} taken"))
}

/// `JobKind::OperatorOwnershipStats`; row counts tell an empty aggregate from a silent failure
pub async fn refresh_operator_ownership(ctx: &AppContext) -> Result<String, AppError> {
    let mut conn = ctx.db.conn().await?;
    stats::refresh_operator_stats(&mut conn).await?;
    let c = stats::operator_stats_counts(&mut conn).await?;
    drop(conn);
    ctx.cache.invalidate_prefix("operators:ownership:").await;
    ctx.cache.invalidate_prefix("operators:buildstats:").await;
    Ok(format!(
        "{} ownership rows ({} E2 owners), {} skill-choice rows, {} module-choice rows",
        c.ownership_rows, c.e2_owners, c.skill_choice_rows, c.module_choice_rows
    ))
}

/// `JobKind::MedalOwnershipStats`: at most one a UTC day unless `force` (a manual refresh)
pub async fn refresh_medal_ownership(ctx: &AppContext, force: bool) -> Result<String, AppError> {
    let mut conn = ctx.db.conn().await?;
    if !force
        && same_utc_day(
            stats::medal_ownership_refreshed_at(&mut conn).await?,
            Utc::now(),
        )
    {
        return Ok("medal ownership already refreshed today".to_owned());
    }
    stats::refresh_medal_ownership(&mut conn).await?;
    let rows = stats::medal_ownership_rows(&mut conn).await?;
    Ok(format!("{rows} medal ownership rows"))
}
