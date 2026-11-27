use myrtle_app::AppContext;
use myrtle_app::services::{grading, sync};
use myrtle_jobs::Job;
use myrtle_types::UserId;
use serde::Deserialize;

use super::{Summary, payload};

#[derive(Deserialize)]
struct Target {
    user_id: UserId,
    #[serde(default)]
    after_sync: bool,
}

pub async fn regrade_all(ctx: &AppContext) -> Summary {
    let queued = grading::enqueue_regrade_all(ctx).await?;
    Ok(format!("{queued} regrade jobs queued"))
}

pub async fn regrade_user(ctx: &AppContext, job: &Job) -> Summary {
    let Target {
        user_id,
        after_sync,
    } = payload(job)?;
    let graded = grading::regrade(ctx, user_id).await;
    if after_sync {
        sync::after_grade(ctx, user_id).await;
    }
    Ok(format!("regraded to {}", graded?.overall))
}
