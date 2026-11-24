use myrtle_app::AppContext;
use myrtle_app::services::music::stats;

use super::Summary;

pub async fn music_stat_dedupe(ctx: &AppContext) -> Summary {
    Ok(stats::sweep_dedupe(ctx).await?)
}
