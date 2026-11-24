use chrono::Utc;
use myrtle_app::services::catalog::dps::watch;
use myrtle_app::services::catalog::maintenance;
use myrtle_app::services::gacha::{pools, shops};
use myrtle_app::services::music::siren;
use myrtle_app::services::sync::sandbox::{self, SandboxBackfill};
use myrtle_app::{AppContext, AppError, alerts};
use myrtle_jobs::{Job, JobKind, JobSpec};
use serde::Deserialize;
use serde_json::json;

use super::{Summary, payload, server};

#[derive(Deserialize)]
struct Refusal {
    #[serde(default)]
    error: String,
}

pub async fn gacha_detail(ctx: &AppContext, job: &Job) -> Summary {
    let fetched = pools::refresh_pool_details(ctx, job.server).await?;
    Ok(format!("{fetched} pool details fetched"))
}

pub async fn event_shop(ctx: &AppContext, job: &Job) -> Summary {
    let fetched = shops::refresh_event_shops(ctx, job.server).await?;
    Ok(format!("{fetched} event shops fetched"))
}

pub async fn maintenance(ctx: &AppContext) -> Summary {
    let next = maintenance::poll(ctx).await?;
    if next.is_zero() {
        return Ok("maintenance poll is off".to_owned());
    }
    if next.as_secs() < ctx.config.jobs.maintenance.poll_secs {
        let sooner = JobSpec {
            dedupe_key: Some("schedule:".to_owned()),
            run_at: Some(Utc::now() + next),
            ..JobSpec::new(JobKind::MaintenancePoll)
        };
        ctx.queue.enqueue(sooner).await.map_err(AppError::from)?;
    }
    Ok(format!("next poll in {}s", next.as_secs()))
}

pub async fn siren(ctx: &AppContext) -> Summary {
    Ok(siren::sync(ctx).await?)
}

pub async fn dps_watch(ctx: &AppContext) -> Summary {
    Ok(watch::watch_upstream(ctx).await?)
}

pub async fn sandbox_backfill(ctx: &AppContext, job: &Job) -> Summary {
    let backfill: SandboxBackfill = payload(job)?;
    let stored = sandbox::backfill(ctx, &backfill).await?;
    Ok(format!("{stored} RA topics stored"))
}

pub async fn service_account_alert(ctx: &AppContext, job: &Job) -> Summary {
    let server = server(job)?.as_str();
    let Refusal { error } = payload(job)?;
    let embed = json!({
        "title": format!("Service account login refused ({server})"),
        "description": format!(
            "The stored `{server}` session no longer refreshes, so its pool details, event \
             shops and probes are stalled.\n\nLog it in again with `myrtle session`.\n\n\
             Upstream said: `{}`",
            error.replace('`', "'")
        ),
        "color": 0x00D9_3F3F_u32,
    });
    let webhook = ctx.secrets.discord_alert_webhook.as_deref();
    alerts::post(ctx.upstream.client(), webhook, embed).await;
    Ok(format!("{server}: refusal reported"))
}
