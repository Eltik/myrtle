mod gamedata;
mod grading;
mod housekeeping;
mod stats;
mod upstream;

use myrtle_app::{AppContext, AppError};
use myrtle_jobs::{Job, JobKind};
use myrtle_types::Server;
use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error(transparent)]
    App(#[from] AppError),
    #[error("bad payload: {0}")]
    Payload(String),
}

pub type Summary = Result<String, HandlerError>;

pub async fn run(ctx: &AppContext, job: &Job) -> Summary {
    match job.kind {
        JobKind::RegradeAll => grading::regrade_all(ctx).await,
        JobKind::RegradeUser => grading::regrade_user(ctx, job).await,
        JobKind::TrendingRecompute => stats::trending(ctx).await,
        JobKind::LeaderboardSnapshot => stats::leaderboard_snapshot(ctx).await,
        JobKind::OperatorOwnershipStats => stats::operator_ownership(ctx).await,
        JobKind::MedalOwnershipStats => stats::medal_ownership(ctx).await,
        JobKind::GachaDetailRefresh => upstream::gacha_detail(ctx, job).await,
        JobKind::EventShopRefresh => upstream::event_shop(ctx, job).await,
        JobKind::MaintenancePoll => upstream::maintenance(ctx).await,
        JobKind::SirenSync => upstream::siren(ctx).await,
        JobKind::DpsUpstreamWatch => upstream::dps_watch(ctx).await,
        JobKind::SandboxBackfill => upstream::sandbox_backfill(ctx, job).await,
        JobKind::ServiceAccountAlert => upstream::service_account_alert(ctx, job).await,
        JobKind::RarityReconcile => gamedata::rarity_reconcile(ctx, job).await,
        JobKind::StoryCommunityAggregate => gamedata::story_community(ctx, job).await,
        JobKind::ReleaseLedgerRecord => gamedata::release_ledger(ctx, job).await,
        JobKind::ReleaseArtMirror => gamedata::release_art_mirror(ctx, job).await,
        JobKind::MusicStatDedupeSweep => housekeeping::music_stat_dedupe(ctx).await,
    }
}

fn payload<T: DeserializeOwned>(job: &Job) -> Result<T, HandlerError> {
    T::deserialize(&job.payload).map_err(|e| HandlerError::Payload(e.to_string()))
}

fn server(job: &Job) -> Result<Server, HandlerError> {
    job.server
        .ok_or_else(|| HandlerError::Payload("the job names no server".into()))
}
