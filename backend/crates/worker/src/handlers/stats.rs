use myrtle_app::AppContext;
use myrtle_app::services::leaderboard::jobs;

use super::Summary;

pub async fn trending(ctx: &AppContext) -> Summary {
    Ok(jobs::recompute_trending(ctx).await?)
}

pub async fn leaderboard_snapshot(ctx: &AppContext) -> Summary {
    Ok(jobs::snapshot_leaderboard(ctx, false).await?)
}

pub async fn operator_ownership(ctx: &AppContext) -> Summary {
    Ok(jobs::refresh_operator_ownership(ctx).await?)
}

pub async fn medal_ownership(ctx: &AppContext) -> Summary {
    Ok(jobs::refresh_medal_ownership(ctx, false).await?)
}
