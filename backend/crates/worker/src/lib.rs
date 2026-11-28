pub mod handlers;
pub mod runtime;

use std::path::Path;
use std::time::Duration;

use myrtle_app::config::Config;
use myrtle_app::context::{BootStep, GameData};
use myrtle_app::services::catalog::release;
use myrtle_app::services::story::community;
use myrtle_app::{AppContext, AppError};
use myrtle_jobs::schedule::{self, Schedule};
use myrtle_jobs::{JobKind, PgQueue};
use myrtle_types::Server;

pub use runtime::Runtime;

pub async fn run(config: Option<&Path>) -> Result<(), AppError> {
    let config = Config::load(config).map_err(AppError::internal)?;
    let ctx = AppContext::open(config, GameData::All, |step| {
        if let BootStep::GameData(server) = step {
            tracing::info!(server = server.as_str(), "loading game data");
        }
    })
    .await?;
    if !ctx.cache.is_redis() {
        tracing::warn!("no redis: maintenance state and cache invalidations stay in this process");
    }

    let schedules = schedules(&ctx);
    let stagger = Duration::from_secs(ctx.config.jobs.stagger_secs);
    schedule::sync(&mut *ctx.db.conn().await?, &schedules, stagger).await?;
    let boot_jobs = release::jobs::enqueue_boot_pass(&ctx).await?;
    tracing::info!(schedules = schedules.len(), boot_jobs, "worker started");

    let queue = PgQueue::new(ctx.db.clone());
    Runtime::new(ctx, queue).run(shutdown_signal()).await
}

#[must_use]
pub fn schedules(ctx: &AppContext) -> Vec<Schedule> {
    let jobs = &ctx.config.jobs;
    let configured = schedule::DEFAULTS.iter().map(|default| {
        let every = match default.kind {
            JobKind::TrendingRecompute => jobs.trending.every_secs,
            JobKind::RegradeAll => jobs.regrade.every_secs,
            JobKind::GachaDetailRefresh => jobs.gacha_detail.every_secs,
            JobKind::EventShopRefresh => jobs.event_shop.every_secs,
            JobKind::MaintenancePoll => jobs.maintenance.poll_secs,
            JobKind::SirenSync => jobs.siren.every_secs,
            JobKind::DpsUpstreamWatch => jobs.dps_watch.every_secs,
            _ => return *default,
        };
        Schedule {
            every: Duration::from_secs(every),
            enabled: every > 0,
            ..*default
        }
    });
    let per_server = ctx
        .snapshots
        .loaded()
        .into_iter()
        .map(story_community_schedule);
    configured.chain(per_server).collect()
}

#[must_use]
pub const fn story_community_schedule(server: Server) -> Schedule {
    Schedule {
        server: Some(server),
        ..Schedule::new(JobKind::StoryCommunityAggregate, community::REFRESH_EVERY)
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
