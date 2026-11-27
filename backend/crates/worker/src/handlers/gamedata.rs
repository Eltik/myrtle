use myrtle_app::AppContext;
use myrtle_app::context::Announced;
use myrtle_app::services::catalog::release;
use myrtle_app::services::gacha::rarity;
use myrtle_app::services::story::community;
use myrtle_jobs::Job;

use super::{Summary, payload, server};

pub async fn rarity_reconcile(ctx: &AppContext, job: &Job) -> Summary {
    let Announced { generation } = payload(job)?;
    let sync = rarity::reconcile_rarities(ctx, generation).await?;
    Ok(format!(
        "{} rows corrected across {} of {} char ids",
        sync.rows_updated, sync.fixed_char_ids, sync.distinct_pairs
    ))
}

pub async fn story_community(ctx: &AppContext, job: &Job) -> Summary {
    let server = server(job)?;
    community::aggregate(ctx, server).await?;
    Ok(format!("{}: story community aggregated", server.as_str()))
}

pub async fn release_ledger(ctx: &AppContext, job: &Job) -> Summary {
    let Announced { generation } = payload(job)?;
    Ok(release::jobs::record_ledger(ctx, server(job)?, generation).await?)
}

pub async fn release_art_mirror(ctx: &AppContext, job: &Job) -> Summary {
    let Announced { generation } = payload(job)?;
    Ok(release::jobs::mirror_art(ctx, server(job)?, generation).await?)
}
