use myrtle_hypergryph::UpstreamError;
use myrtle_jobs::{JobKind, JobSpec};
use serde_json::{Value, json};

use crate::AppContext;

const MAX_LISTED: usize = 15;

#[must_use]
pub fn degraded_tables_embed(server: &str, warnings: &[String]) -> Option<Value> {
    if warnings.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = warnings
        .iter()
        .take(MAX_LISTED)
        .map(|w| format!("• `{}`", w.replace('`', "'")))
        .collect();
    if warnings.len() > MAX_LISTED {
        lines.push(format!("…and {} more", warnings.len() - MAX_LISTED));
    }
    Some(json!({
        "title": format!("Degraded game-data tables ({server})"),
        "description": lines.join("\n"),
        "color": 0x00E0_A81A_u32,
        "footer": { "text": "These tables fell back to empty defaults. Data served from them is wrong, not missing." }
    }))
}

pub async fn report_degraded_tables(
    client: &reqwest::Client,
    webhook: Option<&str>,
    server: &str,
    warnings: &[String],
) {
    let Some(embed) = degraded_tables_embed(server, warnings) else {
        return;
    };
    tracing::warn!(
        server,
        count = warnings.len(),
        "game data loaded with degraded tables"
    );
    post(client, webhook, embed).await;
}

#[must_use]
pub fn service_account_refused_embed(server: &str, error: &str) -> Value {
    let login = if server == "bili" {
        "myrtle session sms --phone <phone>\n\
         myrtle session sms --phone <phone> --otp <code>\n\
         # outside mainland China add --country-code <c> to the first; SMS gets past code 200007"
    } else {
        "myrtle session send-code --server <server> --email <addr>\n\
         myrtle session login --server <server> --email <addr> --code <code>"
    }
    .replace("<server>", server);
    json!({
        "title": format!("Service account login refused ({server})"),
        "description": format!(
            "The stored `{server}` session no longer refreshes, so its pool details, \
             event shops and probes are stalled.\n\n\
             Log it in again:\n```\n{login}\n```\n\
             The next attempt adopts the new credential; no restart needed.\n\n\
             Upstream said: `{}`",
            error.replace('`', "'")
        ),
        "color": 0x00D0_3A3A_u32,
    })
}

/// Queues the Discord alert on the first refusal of an outage; any other error is ignored
pub async fn service_account_refused(ctx: &AppContext, error: &UpstreamError) {
    let UpstreamError::ServiceAccountRefused {
        server,
        first_of_outage: true,
        reason,
    } = error
    else {
        return;
    };
    let spec = JobSpec {
        server: Some(*server),
        payload: json!({ "error": reason }),
        dedupe_key: Some(format!("refused:{}", server.as_str())), // other processes' first refusals fold into the queued one
        ..JobSpec::new(JobKind::ServiceAccountAlert)
    };
    if let Err(e) = ctx.queue.enqueue(spec).await {
        tracing::error!(server = server.as_str(), error = %e, "service account alert not queued");
    }
}

pub async fn post(client: &reqwest::Client, webhook: Option<&str>, embed: Value) {
    let Some(webhook) = webhook.filter(|w| !w.trim().is_empty()) else {
        tracing::warn!(title = %embed["title"], "secrets.discord_alert_webhook unset, alert dropped");
        return;
    };
    match client
        .post(webhook)
        .json(&json!({ "embeds": [embed] }))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {}
        Ok(resp) => tracing::error!(status = %resp.status(), "discord alert rejected"),
        Err(e) => tracing::error!(error = %e, "discord alert failed to send"),
    }
}
